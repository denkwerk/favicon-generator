//! Checks the generated images themselves: sizes, transparency and ico frames.

mod common;

use common::Project;

fn open(project: &Project, file: &str) -> image::DynamicImage {
    image::open(project.path(file)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

/// The `NxN` part of a file name; 180 for `apple-touch-icon(-precomposed).png`.
fn size_in_name(file: &str) -> u32 {
    file.trim_end_matches(".png")
        .split('-')
        .find_map(|part| {
            part.split_once('x')
                .and_then(|(w, h)| (w == h).then(|| w.parse().ok()).flatten())
        })
        .unwrap_or(180)
}

#[test]
fn pngs_have_the_size_in_their_name() {
    let project = Project::new();
    project.run(&[
        "-i",
        "logo.svg",
        "-o",
        "out",
        "--legacy",
        "--manifest",
        "--maskable",
        "--windows",
    ]);
    let pngs: Vec<String> = project
        .files("out")
        .into_iter()
        .filter(|f| f.ends_with(".png"))
        .collect();
    assert!(pngs.len() > 25, "{pngs:?}");
    for file in pngs {
        let image = open(&project, &format!("out/{file}"));
        let size = size_in_name(&file);
        assert_eq!((image.width(), image.height()), (size, size), "{file}");
    }
}

#[test]
fn touch_and_maskable_icons_are_opaque() {
    let project = Project::new();
    std::fs::write(
        project.path("dot.svg"),
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><circle cx="5" cy="5" r="3" fill="#f00"/></svg>"##,
    )
    .unwrap();
    project.run(&[
        "-i",
        "dot.svg",
        "-o",
        "out",
        "--apple-touch-background",
        "#0000ff",
        "--maskable",
        "--background-color",
        "#00ff00",
    ]);

    let apple = open(&project, "out/apple-touch-icon.png");
    assert!(
        !apple.color().has_alpha(),
        "apple-touch-icon.png has an alpha channel"
    );
    assert_eq!(
        apple.to_rgb8().get_pixel(0, 0).0,
        [0, 0, 255],
        "corner shows the background"
    );

    let maskable = open(&project, "out/favicon-maskable-512x512.png").to_rgb8();
    assert_eq!(
        maskable.get_pixel(0, 0).0,
        [0, 255, 0],
        "corner shows the background"
    );
    assert_eq!(
        maskable.get_pixel(256, 256).0,
        [255, 0, 0],
        "the image is in the middle"
    );

    let icon = open(&project, "out/favicon-96x96.png").to_rgba8();
    assert_eq!(
        icon.get_pixel(0, 0).0[3],
        0,
        "favicon PNGs keep their transparency"
    );
}

#[test]
fn ico_contains_the_expected_frames() {
    let frames = |args: &[&str]| {
        let project = Project::new();
        let mut all = vec!["-i", "logo.svg", "-o", "out"];
        all.extend_from_slice(args);
        project.run(&all);
        let file = std::fs::File::open(project.path("out/favicon.ico")).unwrap();
        let mut sizes: Vec<u32> = ico::IconDir::read(file)
            .unwrap()
            .entries()
            .iter()
            .map(|e| e.width())
            .collect();
        sizes.sort();
        sizes
    };
    assert_eq!(frames(&[]), [16, 32, 48]);
    assert_eq!(frames(&["--legacy"]), [16, 24, 32, 48, 64, 128, 256]);
}
