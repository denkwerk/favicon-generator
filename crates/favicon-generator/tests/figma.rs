//! The Figma export against a local mock of the REST API.

mod common;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc;

use common::{LOGO, Project};

type Requests = mpsc::Receiver<(String, Option<String>)>;

/// A mock server answering one request per response, in order. `responses`
/// gets the server's base URL, so a response can point back at it. Reports
/// each request's first line and `X-Figma-Token` header.
fn mock(responses: impl FnOnce(&str) -> Vec<(u16, String)>) -> (String, Requests) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let responses = responses(&base);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut token = None;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                if let Some((_, value)) = line
                    .split_once(':')
                    .filter(|(k, _)| k.eq_ignore_ascii_case("x-figma-token"))
                {
                    token = Some(value.trim().to_owned());
                }
            }
            tx.send((request_line.trim().to_owned(), token)).unwrap();
            write!(
                stream,
                "HTTP/1.1 {status} X\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
    });
    (base, rx)
}

const LINK: &str = "https://www.figma.com/design/FILEKEY/Assets?node-id=12-34";

fn run_against(project: &Project, base: &str, token: &str) -> std::process::Output {
    project
        .command()
        .args(["-i", LINK, "-o", "out"])
        .env("FAVICON_GENERATOR_FIGMA_API", format!("{base}/v1"))
        .env("FIGMA_TOKEN", token)
        .output()
        .unwrap()
}

#[test]
fn exports_the_node_as_svg() {
    // The images API answers with a download URL, served by the same mock.
    let (base, requests) = mock(|base| {
        vec![
            (
                200,
                format!(r#"{{ "err": null, "images": {{ "12:34": "{base}/download.svg" }} }}"#),
            ),
            (200, LOGO.to_owned()),
        ]
    });
    let project = Project::new();
    let output = run_against(&project, &base, "secret-token");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (images, token) = requests.recv().unwrap();
    assert!(images.starts_with("GET /v1/images/FILEKEY?"), "{images}");
    assert!(
        images.contains("ids=12%3A34") && images.contains("format=svg"),
        "{images}"
    );
    assert_eq!(token.as_deref(), Some("secret-token"));
    let (download, token) = requests.recv().unwrap();
    assert!(download.starts_with("GET /download.svg"), "{download}");
    assert_eq!(
        token, None,
        "the token must not be sent to the download URL"
    );

    assert_eq!(project.read("out/favicon.svg"), LOGO);
}

#[test]
fn reports_api_errors_with_a_hint() {
    let (base, _requests) = mock(|_| {
        vec![(
            403,
            r#"{ "status": 403, "err": "Invalid token" }"#.to_owned(),
        )]
    });
    let project = Project::new();
    let output = run_against(&project, &base, "wrong");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("Invalid token") && stderr.contains("check that the token is valid"),
        "{stderr}"
    );
}

#[test]
fn requires_a_token() {
    let project = Project::new();
    let stderr = project.fail(&["-i", LINK, "-o", "out"]);
    assert!(
        stderr.contains("a Figma access token is required"),
        "{stderr}"
    );
}

fn nodes_response(version: &str) -> (u16, String) {
    (
        200,
        format!(r#"{{ "version": "{version}", "nodes": {{ "12:34": {{ "document": {{}} }} }} }}"#),
    )
}

fn export_responses(base: &str, svg: &str) -> [(u16, String); 2] {
    [
        (
            200,
            format!(r#"{{ "err": null, "images": {{ "12:34": "{base}/download.svg" }} }}"#),
        ),
        (200, svg.to_owned()),
    ]
}

/// Runs against the mock with a cache and returns the request paths and stderr.
fn run_cached(project: &Project, base: &str, requests: &Requests) -> (Vec<String>, String) {
    let output = project
        .command()
        .args(["-i", LINK, "-o", "out", "-y", "--cache-dir", "cache"])
        .env("FAVICON_GENERATOR_FIGMA_API", format!("{base}/v1"))
        .env("FIGMA_TOKEN", "secret-token")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    let paths = requests
        .try_iter()
        .map(|(line, _)| line.split(['?', ' ']).nth(1).unwrap().to_owned())
        .collect();
    (paths, stderr + &String::from_utf8_lossy(&output.stdout))
}

#[test]
fn reuses_the_export_while_the_file_is_unchanged() {
    let (base, requests) = mock(|base| {
        let mut responses = vec![nodes_response("1")];
        responses.extend(export_responses(base, LOGO));
        responses.push(nodes_response("1"));
        responses
    });
    let project = Project::new();

    let (paths, _) = run_cached(&project, &base, &requests);
    assert_eq!(
        paths,
        [
            "/v1/files/FILEKEY/nodes",
            "/v1/images/FILEKEY",
            "/download.svg"
        ]
    );

    // One small request instead of an export and a download.
    let (paths, output) = run_cached(&project, &base, &requests);
    assert_eq!(paths, ["/v1/files/FILEKEY/nodes"]);
    assert!(
        output.contains("is unchanged; reusing the export"),
        "{output}"
    );
    assert!(output.contains("from the cache"), "{output}");
    assert_eq!(project.read("out/favicon.svg"), LOGO);
}

#[test]
fn exports_again_when_the_file_changed() {
    let changed = LOGO.replace("<svg", "<svg data-changed=\"1\"");
    let (base, requests) = mock(|base| {
        let mut responses = vec![nodes_response("1")];
        responses.extend(export_responses(base, LOGO));
        // Another part of the file changed; the node looks the same.
        responses.push(nodes_response("2"));
        responses.extend(export_responses(base, LOGO));
        // The node itself changed.
        responses.push(nodes_response("3"));
        responses.extend(export_responses(base, &changed));
        responses
    });
    let project = Project::new();
    run_cached(&project, &base, &requests);

    let (paths, output) = run_cached(&project, &base, &requests);
    assert_eq!(paths.len(), 3, "{paths:?}");
    // The same SVG renders to the same files.
    assert!(output.contains("from the cache"), "{output}");

    let (paths, output) = run_cached(&project, &base, &requests);
    assert_eq!(paths.len(), 3, "{paths:?}");
    assert!(!output.contains("from the cache"), "{output}");
    assert_eq!(project.read("out/favicon.svg"), changed);
}

#[test]
fn falls_back_to_the_cached_export_when_figma_fails() {
    let (base, requests) = mock(|base| {
        let mut responses = vec![nodes_response("1")];
        responses.extend(export_responses(base, LOGO));
        responses.push((
            429,
            r#"{ "status": 429, "err": "Rate limit exceeded" }"#.into(),
        ));
        responses
    });
    let project = Project::new();
    run_cached(&project, &base, &requests);

    let (_, output) = run_cached(&project, &base, &requests);
    assert!(
        output.contains("could not check Figma for changes, reusing the export from just now")
            && output.contains("Rate limit exceeded"),
        "{output}"
    );
    assert_eq!(project.read("out/favicon.svg"), LOGO);

    // Without a token, too.
    let output = project.run(&["-i", LINK, "-o", "out", "-y", "--cache-dir", "cache"]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("a Figma access token is required"));
}

#[test]
fn does_not_check_the_version_without_a_cache() {
    let (base, requests) = mock(|base| export_responses(base, LOGO).into());
    let project = Project::new();
    std::fs::create_dir(project.path("node_modules")).unwrap();
    let output = project
        .command()
        .args(["-i", LINK, "-o", "out", "--no-cache"])
        .env("FAVICON_GENERATOR_FIGMA_API", format!("{base}/v1"))
        .env("FIGMA_TOKEN", "secret-token")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(requests.try_iter().count(), 2);
    assert!(!project.path("node_modules/.cache").exists());
}
