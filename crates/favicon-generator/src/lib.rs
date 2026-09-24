//! favicon-generator: favicons, touch icons, a web app manifest and head tags
//! from one image. The binary in `main.rs` is a thin wrapper around [`run`].

pub mod assets;
pub mod cache;
pub mod cli;
pub mod config;
pub mod figma;
pub mod generate;
pub mod source;
pub mod spec;

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use anyhow::{Context, Result, bail};

use cache::{Cache, Input};
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
    let cache = settings
        .cache
        .then(|| {
            settings
                .cache_dir
                .clone()
                .or_else(|| Cache::default_dir(&cwd))
        })
        .flatten()
        .map(Cache::new);
    if let Some(manifest) = &settings.manifest
        && manifest.name.is_none()
        && manifest.short_name.is_none()
    {
        eprintln!(
            "warning: the manifest has no name or short name; browsers need one to offer installing the app"
        );
    }

    let input = read_input(&settings, cache.as_ref())?;
    let key = cache.as_ref().and_then(|_| cache::key(&settings, &input));
    let cached = cache
        .as_ref()
        .zip(key.as_deref())
        .and_then(|(cache, key)| cache.outputs(key));
    let from_cache = cached.is_some();
    let files = match cached {
        Some(files) => files,
        None => {
            let source = decode(&settings, input)?;
            let files = generate::generate(&settings, &source)?;
            if let Some((cache, key)) = cache.as_ref().zip(key.as_deref())
                && let Err(error) = cache.store_outputs(key, &files)
            {
                eprintln!("warning: could not write the cache: {error:#}");
            }
            files
        }
    };
    write_files(&settings.output, &files, settings.overwrite)?;
    println!(
        "Wrote {} files to {}{}",
        files.len(),
        settings.output.display(),
        if from_cache {
            " (unchanged, from the cache)"
        } else {
            ""
        }
    );
    Ok(())
}

/// Reads the input file, or exports it from Figma. With a cache, a Figma
/// export is reused while the Figma file's version is unchanged.
fn read_input(settings: &Settings, cache: Option<&Cache>) -> Result<Input> {
    if !is_figma_url(&settings.input) {
        let path = Path::new(&settings.input);
        return Ok(Input {
            bytes: fs::read(path).with_context(|| format!("failed to read {}", path.display()))?,
            is_svg: path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("svg")),
        });
    }

    let node = figma::NodeRef::parse_url(&settings.input)?;
    let cached = cache.and_then(|cache| cache.figma_export(&node));
    let token = figma::resolve_token(
        settings.figma_token.as_deref(),
        settings.figma_token_file.as_deref(),
    );
    let svg = match (cache, cached) {
        (None, _) => {
            eprintln!(
                "Exporting node {} from Figma file {}…",
                node.node_id, node.file_key
            );
            figma::fetch_svg(&node, &token?)?
        }
        (Some(cache), cached) => {
            let version = token
                .as_deref()
                .map_err(|e| anyhow::anyhow!("{e:#}"))
                .and_then(|token| figma::file_version(&node, token));
            match (version, cached) {
                (Ok(version), Some(cached)) if version == cached.version => {
                    eprintln!(
                        "Figma file {} is unchanged; reusing the export of node {}",
                        node.file_key, node.node_id
                    );
                    cached.svg
                }
                (Ok(version), _) => {
                    eprintln!(
                        "Exporting node {} from Figma file {}…",
                        node.node_id, node.file_key
                    );
                    let svg = figma::fetch_svg(&node, &token?)?;
                    if let Err(error) = cache.store_figma_export(&node, &version, &svg) {
                        eprintln!("warning: could not write the cache: {error:#}");
                    }
                    svg
                }
                // Offline, rate limited or without a token: an older export beats failing.
                (Err(error), Some(cached)) => {
                    eprintln!(
                        "warning: could not check Figma for changes, reusing the export from {}: {error:#}",
                        age(cached.exported)
                    );
                    cached.svg
                }
                (Err(error), None) => return Err(error),
            }
        }
    };
    Ok(Input {
        bytes: svg,
        is_svg: true,
    })
}

/// "5 minutes ago", roughly.
fn age(time: SystemTime) -> String {
    let secs = SystemTime::now()
        .duration_since(time)
        .unwrap_or_default()
        .as_secs();
    let (n, unit) = match secs {
        0..60 => return "just now".into(),
        60..3600 => (secs / 60, "minute"),
        3600..86400 => (secs / 3600, "hour"),
        _ => (secs / 86400, "day"),
    };
    format!("{n} {unit}{} ago", if n == 1 { "" } else { "s" })
}

/// Parses the input and warns about sources that make poor icons.
fn decode(settings: &Settings, input: Input) -> Result<Source> {
    let source = if is_figma_url(&settings.input) {
        Source::from_svg(input.bytes, None)
            .context("Figma returned an SVG that could not be parsed")?
    } else {
        Source::decode(input.bytes, input.is_svg, Path::new(&settings.input))?
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
