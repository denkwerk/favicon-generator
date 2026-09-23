//! favicon-generator: favicons, touch icons, a web app manifest and head tags
//! from one image. The binary in `main.rs` is a thin wrapper around [`run`].

pub mod assets;
pub mod cli;
pub mod config;
pub mod figma;
pub mod generate;
pub mod source;
pub mod spec;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use cli::{Cli, is_figma_url};
use config::{FileConfig, Settings};
use generate::OutputFile;
use source::Source;

/// Runs the CLI: loads the config, merges the flags, generates and writes the files.
pub fn run(cli: Cli) -> Result<()> {
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
    let source = load_source(&settings)?;
    if let Some(manifest) = &settings.manifest
        && manifest.name.is_none()
        && manifest.short_name.is_none()
    {
        eprintln!(
            "warning: the manifest has no name or short name; browsers need one to offer installing the app"
        );
    }
    let files = generate::generate(&settings, &source)?;
    write_files(&settings.output, &files, settings.overwrite)?;
    println!(
        "Wrote {} files to {}",
        files.len(),
        settings.output.display()
    );
    Ok(())
}

fn load_source(settings: &Settings) -> Result<Source> {
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
    Ok(source)
}

fn write_files(dir: &Path, files: &[OutputFile], overwrite: bool) -> Result<()> {
    if !overwrite {
        let existing: Vec<&str> = files
            .iter()
            .map(|file| file.name.as_str())
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
    for file in files {
        let path = dir.join(&file.name);
        fs::write(&path, &file.data)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}
