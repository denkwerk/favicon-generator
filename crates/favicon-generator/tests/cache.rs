//! Reusing generated files from the cache. Figma exports are covered in figma.rs.

mod common;

use std::time::{Duration, Instant};

use common::{LOGO, Project, list};

/// Every option group on, so that many icons are rendered.
const FULL: &[&str] = &[
    "--legacy",
    "--manifest",
    "--maskable",
    "--windows",
    "--name",
    "Cached",
];

fn from_cache(output: &std::process::Output) -> bool {
    String::from_utf8_lossy(&output.stdout).contains("(unchanged, from the cache)")
}

/// Every file in `dir` with its contents.
fn snapshot(project: &Project, dir: &str) -> Vec<(String, Vec<u8>)> {
    project
        .files(dir)
        .into_iter()
        .map(|name| {
            let data = std::fs::read(project.path(dir).join(&name)).unwrap();
            (name, data)
        })
        .collect()
}

#[test]
fn reuses_the_generated_files() {
    let project = Project::new();
    let args = |out: &'static str| {
        let mut args = vec!["logo.svg", out, "--cache-dir", "cache"];
        args.extend(FULL);
        args
    };

    assert!(!from_cache(&project.run(&args("first"))));
    assert!(from_cache(&project.run(&args("second"))));

    let mut uncached = args("uncached");
    uncached.retain(|arg| !["--cache-dir", "cache"].contains(arg));
    uncached.push("--no-cache");
    assert!(!from_cache(&project.run(&uncached)));

    // Byte for byte what generating produces.
    let first = snapshot(&project, "first");
    assert_eq!(first.len(), 32);
    assert_eq!(snapshot(&project, "second"), first);
    assert_eq!(snapshot(&project, "uncached"), first);
}

#[test]
fn regenerates_when_the_input_or_the_options_change() {
    let project = Project::new();
    let run = |args: &[&str]| {
        let mut all = vec!["logo.svg", "-y", "--cache-dir", "cache"];
        all.extend(args);
        if !args.contains(&"-o") {
            all.extend(["-o", "out"]);
        }
        from_cache(&project.run(&all))
    };
    assert!(!run(&[]));
    assert!(run(&[]));

    assert!(!run(&["--theme-color", "#123456"]));
    assert!(!run(&["--path-prefix", "/icons/"]));
    // Where the files go does not matter.
    assert!(run(&["--theme-color", "#123456", "-o", "elsewhere"]));

    project.write("logo.svg", LOGO.replace("<svg", "<svg data-changed=\"1\""));
    assert!(!run(&[]));
    assert!(project.read("out/favicon.svg").contains("data-changed"));

    // Earlier entries are kept.
    project.write("logo.svg", LOGO);
    assert!(run(&[]));
    assert_eq!(project.read("out/favicon.svg"), LOGO);
    assert_eq!(list(&project.path("cache/outputs")).len(), 4);
}

#[test]
fn defaults_to_node_modules_cache_in_the_project() {
    let project = Project::new();
    project.run(&["logo.svg", "out"]);
    assert!(
        !project.path("node_modules").exists(),
        "no node_modules, no cache"
    );

    std::fs::create_dir(project.path("node_modules")).unwrap();
    project.run(&["logo.svg", "out", "-y", "--no-cache"]);
    assert!(!project.path("node_modules/.cache").exists());

    project.run(&["logo.svg", "out", "-y"]);
    assert!(from_cache(&project.run(&["logo.svg", "out", "-y"])));
    assert_eq!(
        list(&project.path("node_modules/.cache/favicon-generator/outputs")).len(),
        1
    );
}

#[test]
fn the_config_file_can_move_or_disable_the_cache() {
    let project = Project::new();
    project.write(
        "favicon.config.json",
        r#"{ "input": "logo.svg", "overwrite": true, "cacheDir": "tmp/favicons" }"#,
    );
    project.run(&[]);
    assert!(from_cache(&project.run(&[])));
    assert!(project.path("tmp/favicons/outputs").is_dir());

    project.write(
        "favicon.config.json",
        r#"{ "input": "logo.svg", "overwrite": true, "cacheDir": "tmp/favicons", "cache": false }"#,
    );
    assert!(!from_cache(&project.run(&[])));
}

#[test]
fn does_not_cache_svgs_with_external_images() {
    let project = Project::new();
    project.write(
        "photo.svg",
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><image href="photo.png" width="10" height="10"/></svg>"#,
    );
    project.run(&["photo.svg", "out", "--cache-dir", "cache"]);
    assert!(!from_cache(&project.run(&[
        "photo.svg",
        "out",
        "-y",
        "--cache-dir",
        "cache"
    ])));
    assert!(!project.path("cache/outputs").exists());
}

/// Median wall time of `runs` runs.
fn median(runs: usize, mut run: impl FnMut()) -> Duration {
    let mut times: Vec<Duration> = (0..runs)
        .map(|_| {
            let start = Instant::now();
            run();
            start.elapsed()
        })
        .collect();
    times.sort();
    times[runs / 2]
}

/// The point of the cache: a hit skips decoding and rendering. Prints the
/// times (`cargo test --test cache -- --nocapture`); see
/// `scripts/bench-cache.ts` for release-build numbers.
#[test]
fn a_cache_hit_is_much_faster() {
    let project = Project::new();
    let run = |cache: &str| {
        let mut args = vec!["logo.svg", "out", "-y", "--cache-dir", cache];
        args.extend(FULL);
        project.run(&args);
    };
    let mut cold_runs = 0;
    let cold = median(3, || {
        // A fresh cache directory each time.
        cold_runs += 1;
        run(&format!("cold{cold_runs}"));
    });
    run("warm");
    let warm = median(3, || run("warm"));

    let speedup = cold.as_secs_f64() / warm.as_secs_f64();
    eprintln!("cold: {cold:.0?}, cached: {warm:.0?} ({speedup:.1}x faster)");
    assert!(
        speedup > 3.0,
        "a cache hit should be much faster: cold {cold:?}, cached {warm:?}"
    );
}
