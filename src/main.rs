mod assets;
mod figma;
mod source;
mod spec;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ImageBuffer, ImageEncoder, Pixel, PixelWithColorType, RgbaImage};

use assets::Config;
use source::Source;

/// Generate favicons, touch icons, a web app manifest and browserconfig.xml
/// from a single SVG (recommended) or raster image, or from a Figma node.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Source image (SVG, PNG, JPEG or WebP; should be square), or a Figma
    /// link with a `node-id` to export that node as SVG via the REST API.
    input: String,

    /// Directory to write the generated files into.
    #[arg(default_value = "favicons")]
    output: PathBuf,

    /// Overwrite existing files in the output directory.
    #[arg(short = 'y', long)]
    overwrite: bool,

    /// URL prefix the files will be served from, e.g. `/public/`.
    #[arg(short = 'p', long, default_value = "/")]
    path_prefix: String,

    /// Application name used in manifest.json.
    #[arg(short = 'n', long, default_value = "App")]
    app_name: String,

    /// Short application name [default: --app-name].
    #[arg(long)]
    app_short_name: Option<String>,

    /// Application description [default: --app-name].
    #[arg(long)]
    app_description: Option<String>,

    /// Browser UI color (`theme-color`, manifest `theme_color`).
    #[arg(long, default_value = "#ffffff", value_parser = parse_color)]
    theme_color: String,

    /// Splash screen color (manifest `background_color`); also used behind
    /// transparent pixels in the opaque apple-touch-icon*.png files.
    #[arg(long, default_value = "#ffffff", value_parser = parse_color)]
    background_color: String,

    /// Windows tile color [default: --background-color].
    #[arg(long, value_parser = parse_color)]
    tile_color: Option<String>,

    /// Manifest `start_url`.
    #[arg(long, default_value = "/?source=pwa")]
    start_url: String,

    /// Manifest `scope`.
    #[arg(long, default_value = "/")]
    scope: String,

    /// Manifest `display` mode.
    #[arg(long, value_enum, default_value_t = Display::Standalone)]
    display: Display,

    /// Manifest icon `purpose`.
    #[arg(long, default_value = "any maskable")]
    icon_purpose: String,

    /// `crossorigin` attribute for the manifest <link>, e.g. `use-credentials`.
    #[arg(long)]
    manifest_crossorigin: Option<String>,

    /// Figma personal access token (scope `file_content:read`), used when INPUT is a Figma link.
    #[arg(long, env = "FIGMA_TOKEN", hide_env_values = true)]
    figma_token: Option<String>,

    /// Read the Figma token from this file instead.
    #[arg(long, env = "FIGMA_TOKEN_FILE")]
    figma_token_file: Option<PathBuf>,

    /// Head snippets to write next to the icons.
    #[arg(long, value_enum, value_delimiter = ',', default_value = "html,nuxt")]
    snippets: Vec<Snippet>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Display {
    Fullscreen,
    Standalone,
    MinimalUi,
    Browser,
}

#[derive(Clone, Copy, PartialEq, ValueEnum)]
enum Snippet {
    /// favicon.html with <link>/<meta> tags.
    Html,
    /// nuxt-head.ts exporting `faviconHead` for `app.head`.
    Nuxt,
    /// Write no snippets.
    None,
}

fn parse_color(s: &str) -> Result<String, String> {
    rgb(s).map(|_| s.to_ascii_lowercase())
}

/// Parses `#rgb` / `#rrggbb`.
fn rgb(s: &str) -> Result<[u8; 3], String> {
    let hex = s.strip_prefix('#').ok_or("color must start with '#'")?;
    let expanded: String = match hex.len() {
        3 => hex.chars().flat_map(|c| [c, c]).collect(),
        6 => hex.to_owned(),
        _ => return Err("expected #rgb or #rrggbb".into()),
    };
    let channel = |i: usize| {
        u8::from_str_radix(&expanded[i..i + 2], 16).map_err(|_| format!("invalid hex color {s}"))
    };
    Ok([channel(0)?, channel(2)?, channel(4)?])
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let source = if is_figma_url(&cli.input) {
        let node = figma::NodeRef::parse_url(&cli.input)?;
        let token =
            figma::resolve_token(cli.figma_token.as_deref(), cli.figma_token_file.as_deref())?;
        eprintln!(
            "Exporting node {} from Figma file {}…",
            node.node_id, node.file_key
        );
        let svg = figma::fetch_svg(&node, &token)?;
        Source::from_svg(svg, None).context("Figma returned an SVG that could not be parsed")?
    } else {
        Source::load(Path::new(&cli.input))?
    };
    let (w, h) = source.dimensions();
    if (w - h).abs() > f32::EPSILON {
        eprintln!("warning: source is {w}x{h}; it will be centered on a transparent square");
    }
    if matches!(source, Source::Raster(_)) && w.max(h) < 512.0 {
        eprintln!(
            "warning: source is smaller than 512px; large icons will be upscaled (use an SVG for best results)"
        );
    }

    let mut path_prefix = cli.path_prefix.clone();
    if !path_prefix.ends_with('/') {
        path_prefix.push('/');
    }
    let background = rgb(&cli.background_color).expect("validated by clap");
    let cfg = Config {
        path_prefix,
        short_name: cli
            .app_short_name
            .clone()
            .unwrap_or_else(|| cli.app_name.clone()),
        description: cli
            .app_description
            .clone()
            .unwrap_or_else(|| cli.app_name.clone()),
        name: cli.app_name.clone(),
        theme_color: cli.theme_color.clone(),
        tile_color: cli
            .tile_color
            .clone()
            .unwrap_or_else(|| cli.background_color.clone()),
        background_color: cli.background_color.clone(),
        start_url: cli.start_url.clone(),
        scope: cli.scope.clone(),
        display: cli
            .display
            .to_possible_value()
            .expect("no skipped variants")
            .get_name()
            .to_owned(),
        icon_purpose: cli.icon_purpose.clone(),
        manifest_crossorigin: cli.manifest_crossorigin.clone(),
        has_svg: source.svg_bytes().is_some(),
    };

    // Render each distinct size once.
    let sizes: BTreeSet<u32> = spec::PNG_SIZES
        .into_iter()
        .chain(spec::ICO_SIZES)
        .chain(spec::APPLE_TOUCH_FILES.map(|(_, size)| size))
        .collect();
    let renders = sizes
        .into_iter()
        .map(|size| Ok((size, source.render(size)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;

    // Assemble all outputs in memory so nothing is written if a step fails.
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for size in spec::PNG_SIZES {
        files.push((spec::png_name(size), encode_png(&renders[&size])?));
    }
    for (suffix, size) in spec::APPLE_TOUCH_FILES {
        let png = encode_png(&source::flatten(&renders[&size], background))?;
        for name in spec::apple_touch_names(suffix) {
            files.push((name, png.clone()));
        }
    }
    files.push(("favicon.ico".into(), encode_ico(&renders)?));
    if let Some(svg) = source.svg_bytes() {
        files.push(("favicon.svg".into(), svg.to_vec()));
    }
    files.push(("manifest.json".into(), assets::manifest(&cfg).into_bytes()));
    files.push((
        "browserconfig.xml".into(),
        assets::browserconfig(&cfg).into_bytes(),
    ));
    if !cli.snippets.contains(&Snippet::None) {
        if cli.snippets.contains(&Snippet::Html) {
            files.push((
                "favicon.html".into(),
                assets::html_snippet(&cfg).into_bytes(),
            ));
        }
        if cli.snippets.contains(&Snippet::Nuxt) {
            files.push((
                "nuxt-head.ts".into(),
                assets::nuxt_snippet(&cfg).into_bytes(),
            ));
        }
    }

    write_files(&cli.output, &files, cli.overwrite)?;
    println!("Wrote {} files to {}", files.len(), cli.output.display());
    Ok(())
}

fn is_figma_url(input: &str) -> bool {
    let rest = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .unwrap_or(input);
    rest.starts_with("figma.com/") || rest.starts_with("www.figma.com/")
}

fn encode_png<P>(image: &ImageBuffer<P, Vec<u8>>) -> Result<Vec<u8>>
where
    P: Pixel<Subpixel = u8> + PixelWithColorType,
{
    let mut out = Vec::new();
    PngEncoder::new_with_quality(
        Cursor::new(&mut out),
        CompressionType::Best,
        FilterType::Adaptive,
    )
    .write_image(image.as_raw(), image.width(), image.height(), P::COLOR_TYPE)?;
    Ok(out)
}

fn encode_ico(renders: &BTreeMap<u32, RgbaImage>) -> Result<Vec<u8>> {
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in spec::ICO_SIZES {
        let frame = ico::IconImage::from_rgba_data(size, size, renders[&size].as_raw().clone());
        // 32-bit BMP frames for maximum compatibility; PNG for 256px keeps the file small.
        let entry = if size < 256 {
            ico::IconDirEntry::encode_as_bmp(&frame)?
        } else {
            ico::IconDirEntry::encode_as_png(&frame)?
        };
        dir.add_entry(entry);
    }
    let mut out = Vec::new();
    dir.write(BufWriter::new(&mut out))?;
    Ok(out)
}

fn write_files(dir: &Path, files: &[(String, Vec<u8>)], overwrite: bool) -> Result<()> {
    if !overwrite {
        let existing: Vec<&str> = files
            .iter()
            .map(|(name, _)| name.as_str())
            .filter(|name| dir.join(name).exists())
            .collect();
        if !existing.is_empty() {
            bail!(
                "{} file(s) already exist in {} (e.g. {}); pass --overwrite to replace them",
                existing.len(),
                dir.display(),
                existing[0]
            );
        }
    }
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    for (name, data) in files {
        let path = dir.join(name);
        fs::write(&path, data).with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}
