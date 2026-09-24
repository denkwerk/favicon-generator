//! End-to-end tests of the binary: which files and tags each option produces,
//! config files and flags, and error messages.

mod common;

use common::{Project, has_node};

#[test]
fn defaults_are_minimal() {
    let project = Project::new();
    project.run(&["-i", "logo.svg", "-o", "out"]);
    assert_eq!(
        project.files("out"),
        [
            "apple-touch-icon.png",
            "favicon-96x96.png",
            "favicon.html",
            "favicon.ico",
            "favicon.svg"
        ]
    );
    insta::assert_snapshot!("default_html", project.read("out/favicon.html"));
}

#[test]
fn raster_input_has_no_svg_favicon() {
    let project = Project::new();
    image::RgbaImage::from_pixel(600, 600, image::Rgba([2, 150, 156, 255]))
        .save(project.path("logo.png"))
        .unwrap();
    project.run(&["-i", "logo.png", "-o", "out"]);
    assert!(!project.files("out").contains(&"favicon.svg".to_owned()));
    assert!(!project.read("out/favicon.html").contains("image/svg+xml"));
}

#[test]
fn every_option_group() {
    let project = Project::new();
    project.run(&[
        "-i",
        "logo.svg",
        "-o",
        "out",
        "-p",
        "/favicons",
        "--theme-color",
        "#02969c",
        "--name",
        "My App",
        "--short-name",
        "App",
        "--description",
        "An app",
        "--background-color",
        "#0b1f24",
        "--start-url",
        "/?source=pwa",
        "--scope",
        "/",
        "--display",
        "standalone",
        "--maskable",
        "--manifest-crossorigin",
        "use-credentials",
        "--tile-color",
        "#02969c",
        "--snippets",
        "html,json,nuxt",
    ]);
    assert_eq!(
        project.files("out"),
        [
            "apple-touch-icon.png",
            "browserconfig.xml",
            "favicon-144x144.png",
            "favicon-150x150.png",
            "favicon-192x192.png",
            "favicon-310x310.png",
            "favicon-512x512.png",
            "favicon-70x70.png",
            "favicon-96x96.png",
            "favicon-head.json",
            "favicon-maskable-192x192.png",
            "favicon-maskable-512x512.png",
            "favicon.html",
            "favicon.ico",
            "favicon.svg",
            "manifest.json",
            "nuxt-head.ts",
        ]
    );
    insta::assert_snapshot!("full_html", project.read("out/favicon.html"));
    insta::assert_snapshot!("full_manifest", project.read("out/manifest.json"));
    insta::assert_snapshot!("full_browserconfig", project.read("out/browserconfig.xml"));
    insta::assert_snapshot!("full_nuxt_head", project.read("out/nuxt-head.ts"));

    // The JSON snippet holds the same tags as the HTML one.
    let json: serde_json::Value =
        serde_json::from_str(&project.read("out/favicon-head.json")).unwrap();
    let tags = json["link"].as_array().unwrap().len() + json["meta"].as_array().unwrap().len();
    assert_eq!(tags, project.read("out/favicon.html").lines().count());
}

#[test]
fn manifest_only_contains_what_is_configured() {
    let project = Project::new();
    project.run(&["-i", "logo.svg", "-o", "out", "--short-name", "App"]);
    let manifest: serde_json::Value =
        serde_json::from_str(&project.read("out/manifest.json")).unwrap();
    let keys: Vec<&str> = manifest
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["short_name", "icons"]);
    assert!(
        project
            .read("out/favicon.html")
            .contains(r#"<link rel="manifest" href="/manifest.json">"#)
    );
    assert!(!project.read("out/favicon.html").contains("theme-color"));
}

#[test]
fn manifest_lists_the_configured_icon_sizes_with_their_purpose() {
    let project = Project::new();
    project.run(&[
        "-i",
        "logo.svg",
        "-o",
        "out",
        "--manifest-icon-sizes",
        "72,512,144",
        "--manifest-icon-purpose",
        "any maskable",
    ]);
    // Sizes that nothing else generates are rendered for the manifest.
    let files = project.files("out");
    for file in [
        "favicon-72x72.png",
        "favicon-144x144.png",
        "favicon-512x512.png",
    ] {
        assert!(files.iter().any(|f| f == file), "{file} missing: {files:?}");
    }
    assert!(
        !files.iter().any(|f| f == "favicon-192x192.png"),
        "{files:?}"
    );

    let manifest: serde_json::Value =
        serde_json::from_str(&project.read("out/manifest.json")).unwrap();
    let icons: Vec<(String, String, String)> = manifest["icons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|icon| {
            let field = |key: &str| icon[key].as_str().unwrap().to_owned();
            (field("src"), field("sizes"), field("purpose"))
        })
        .collect();
    assert_eq!(
        icons,
        [72, 144, 512].map(|size| (
            format!("/favicon-{size}x{size}.png"),
            format!("{size}x{size}"),
            "any maskable".to_owned()
        ))
    );
}

#[test]
fn legacy_adds_the_old_sizes() {
    let project = Project::new();
    project.run(&["-i", "logo.svg", "-o", "out", "--legacy"]);
    let files = project.files("out");
    assert_eq!(
        files
            .iter()
            .filter(|f| f.starts_with("favicon-") && f.ends_with(".png"))
            .count(),
        19
    );
    for file in [
        "apple-touch-icon-precomposed.png",
        "apple-touch-icon-120x120.png",
        "apple-touch-icon-152x152-precomposed.png",
    ] {
        assert!(files.contains(&file.to_owned()), "missing {file}");
    }
    assert!(
        !files.contains(&"manifest.json".to_owned()),
        "legacy must not imply the manifest"
    );
    insta::assert_snapshot!("legacy_html", project.read("out/favicon.html"));
}

#[test]
fn apple_touch_icon_can_be_turned_off() {
    let project = Project::new();
    project.run(&["-i", "logo.svg", "-o", "out", "--no-apple-touch-icon"]);
    assert!(
        !project
            .files("out")
            .contains(&"apple-touch-icon.png".to_owned())
    );
    assert!(
        !project
            .read("out/favicon.html")
            .contains("apple-touch-icon")
    );
}

#[test]
fn reads_a_json_config_and_lets_flags_override_it() {
    let project = Project::new();
    project.write(
        "favicon.config.json",
        r##"{
            "input": "./logo.svg",
            "output": "./public/favicons",
            "pathPrefix": "/favicons/",
            "themeColor": "#111111",
            "manifest": { "name": "From file", "shortName": "File" }
        }"##,
    );
    let output = project.run(&["--name", "From flag"]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("favicon.config.json"));
    let manifest: serde_json::Value =
        serde_json::from_str(&project.read("public/favicons/manifest.json")).unwrap();
    assert_eq!(manifest["name"], "From flag");
    assert_eq!(manifest["short_name"], "File");
    assert_eq!(manifest["theme_color"], "#111111");
    assert_eq!(manifest["icons"][0]["src"], "/favicons/favicon-192x192.png");
}

#[test]
fn finds_the_config_in_a_parent_directory() {
    let project = Project::new();
    project.write(
        "favicon.config.json",
        r#"{ "input": "logo.svg", "output": "out" }"#,
    );
    project.write("src/nested/.keep", "");
    let output = project
        .command()
        .current_dir(project.path("src/nested"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Paths are relative to the config file, not the working directory.
    assert!(project.path("out/favicon.ico").exists());
}

#[test]
fn evaluates_a_typescript_config_with_node() {
    if !has_node() {
        eprintln!("skipped: Node.js is not installed");
        return;
    }
    let project = Project::new();
    project.write(
        "favicon.config.ts",
        r#"
        const name: string = ['My', 'App'].join(' ')
        export default async () => ({
          input: './logo.svg',
          output: './out',
          manifest: { name },
          windows: true,
        })
        "#,
    );
    project.run(&[]);
    let files = project.files("out");
    assert!(
        files.contains(&"manifest.json".to_owned())
            && files.contains(&"browserconfig.xml".to_owned())
    );
    assert!(
        project
            .read("out/manifest.json")
            .contains(r#""name": "My App""#)
    );
}

#[test]
fn reads_the_config_from_stdin() {
    use std::io::Write;
    let project = Project::new();
    let mut child = project
        .command()
        .args(["--config", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{ "input": "logo.svg", "output": "out", "legacy": true }"#)
        .unwrap();
    assert!(child.wait().unwrap().success());
    assert!(project.path("out/favicon-512x512.png").exists());
}

#[test]
fn prints_the_config() {
    let project = Project::new();
    project.write(
        "favicon.config.json",
        r#"{ "input": "logo.svg", "manifest": true }"#,
    );
    let output = project.run(&["--print-config"]);
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["path"]
            .as_str()
            .unwrap()
            .ends_with("favicon.config.json")
    );
    assert_eq!(json["config"]["manifest"], true);
    assert!(
        json["config"]["input"]
            .as_str()
            .unwrap()
            .ends_with("logo.svg")
    );
}

#[test]
fn ignores_the_config_with_no_config() {
    let project = Project::new();
    project.write("favicon.config.json", r#"{ "input": "missing.svg" }"#);
    project.run(&["-i", "logo.svg", "-o", "out", "--no-config"]);
}

#[test]
fn refuses_to_overwrite_without_the_flag() {
    let project = Project::new();
    project.run(&["-i", "logo.svg", "-o", "out"]);
    let stderr = project.fail(&["-i", "logo.svg", "-o", "out"]);
    assert!(
        stderr.contains("already exist") && stderr.contains("--overwrite"),
        "{stderr}"
    );
    project.run(&["-i", "logo.svg", "-o", "out", "-y"]);
}

#[test]
fn explains_errors() {
    let project = Project::new();
    assert!(project.fail(&["-o", "out"]).contains("no input given"));
    assert!(project.fail(&["-i", "missing.svg"]).contains("missing.svg"));
    assert!(
        project
            .fail(&["logo.svg", "-i", "other.svg"])
            .contains("cannot be used with")
    );
    assert!(
        project
            .fail(&["-i", "logo.svg", "--theme-color", "teal"])
            .contains("color must start with '#'")
    );

    project.write(
        "favicon.config.json",
        r#"{ "input": "logo.svg", "appName": "Old" }"#,
    );
    let stderr = project.fail(&[]);
    assert!(
        stderr.contains("`appName` is now `manifest.name`"),
        "{stderr}"
    );
}

#[test]
fn warns_about_a_manifest_without_a_name() {
    let project = Project::new();
    let output = project.run(&["-i", "logo.svg", "-o", "out", "--manifest"]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("no name or short name"));
}
