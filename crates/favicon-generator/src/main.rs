mod assets;
mod cli;
mod config;
mod figma;
mod source;
mod spec;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufWriter, Cursor};
use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::Parser;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ImageBuffer, ImageEncoder, Pixel, PixelWithColorType, RgbaImage};

use cli::{Cli, Snippet, is_figma_url};
use config::{FileConfig, Settings};
use source::Source;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("failed to read the current directory")?;

    let (path, file_config) = match &cli.config {
        Some(path) if path.as_os_str() == "-" => (None, config::load_stdin(&cwd)?),
        Some(path) => {
            let path = cwd.join(path);
            let config = config::load(&path)?;
            (Some(path), config)
        }
        None if cli.no_config => (None, FileConfig::default()),
        None => match config::discover(&cwd) {
            Some(path) => {
                if !cli.print_config {
                    eprintln!("Using {}", path.display());
                }
                let config = config::load(&path)?;
                (Some(path), config)
            }
            None => (None, FileConfig::default()),
        },
    };

    if cli.print_config {
        let json = serde_json::json!({ "path": path, "config": file_config });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    let settings = Settings::merge(cli, file_config)?;
    generate(&settings)
}

fn generate(settings: &Settings) -> Result<()> {
    let source = if is_figma_url(&settings.input) {
        let node = figma::NodeRef::parse_url(&settings.input)?;
        let token = figma::resolve_token(
            settings.figma_token.as_deref(),
            settings.figma_token_file.as_deref(),
        )?;
        eprintln!(
            "Exporting node {} from Figma file {}…",
            node.node_id, node.file_key
        );
        let svg = figma::fetch_svg(&node, &token)?;
        Source::from_svg(svg, None).context("Figma returned an SVG that could not be parsed")?
    } else {
        Source::load(Path::new(&settings.input))?
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

    let cfg = assets::Config {
        path_prefix: settings.path_prefix.clone(),
        name: settings.app_name.clone(),
        short_name: settings.app_short_name.clone(),
        description: settings.app_description.clone(),
        theme_color: settings.theme_color.clone(),
        background_color: settings.background_color.clone(),
        tile_color: settings.tile_color.clone(),
        start_url: settings.start_url.clone(),
        scope: settings.scope.clone(),
        display: settings.display.as_str().to_owned(),
        icon_purpose: settings.icon_purpose.clone(),
        manifest_crossorigin: settings.manifest_crossorigin.clone(),
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
        let png = encode_png(&source::flatten(&renders[&size], settings.background_rgb))?;
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
    if settings.snippets.contains(&Snippet::Html) {
        files.push((
            "favicon.html".into(),
            assets::html_snippet(&cfg).into_bytes(),
        ));
    }
    if settings.snippets.contains(&Snippet::Json) {
        files.push((
            "favicon-head.json".into(),
            assets::json_snippet(&cfg).into_bytes(),
        ));
    }
    if settings.snippets.contains(&Snippet::Nuxt) {
        files.push((
            "nuxt-head.ts".into(),
            assets::nuxt_snippet(&cfg).into_bytes(),
        ));
    }

    write_files(&settings.output, &files, settings.overwrite)?;
    println!(
        "Wrote {} files to {}",
        files.len(),
        settings.output.display()
    );
    Ok(())
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
