use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use serde::Deserialize;

/// Generate favicons, touch icons, a web app manifest and browserconfig.xml
/// from a single SVG (recommended) or raster image, or from a Figma node.
///
/// Options can also be set in a favicon.config.{js,ts,mjs,mts,cjs,cts,json}
/// file, which is looked up in the current directory and its parents up to the
/// project root (the nearest directory with a package.json or .git).
/// Command-line flags take precedence over the config file.
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Source image (SVG, PNG, JPEG or WebP; should be square), or a Figma
    /// link with a `node-id` to export that node as SVG via the REST API.
    pub input: Option<String>,

    /// Directory to write the generated files into [default: favicons].
    pub output: Option<PathBuf>,

    /// Config file to use instead of searching for one; `-` reads JSON from stdin.
    #[arg(short, long, conflicts_with = "no_config")]
    pub config: Option<PathBuf>,

    /// Ignore favicon.config.* files.
    #[arg(long)]
    pub no_config: bool,

    /// Overwrite existing files in the output directory.
    #[arg(short = 'y', long)]
    pub overwrite: bool,

    /// URL prefix the files will be served from, e.g. `/public/` [default: /].
    #[arg(short = 'p', long)]
    pub path_prefix: Option<String>,

    /// Application name used in manifest.json [default: App].
    #[arg(short = 'n', long)]
    pub app_name: Option<String>,

    /// Short application name [default: --app-name].
    #[arg(long)]
    pub app_short_name: Option<String>,

    /// Application description [default: --app-name].
    #[arg(long)]
    pub app_description: Option<String>,

    /// Browser UI color (`theme-color`, manifest `theme_color`) [default: #ffffff].
    #[arg(long, value_parser = parse_color)]
    pub theme_color: Option<String>,

    /// Splash screen color (manifest `background_color`); also used behind
    /// transparent pixels in the opaque apple-touch-icon*.png files [default: #ffffff].
    #[arg(long, value_parser = parse_color)]
    pub background_color: Option<String>,

    /// Windows tile color [default: --background-color].
    #[arg(long, value_parser = parse_color)]
    pub tile_color: Option<String>,

    /// Manifest `start_url` [default: /?source=pwa].
    #[arg(long)]
    pub start_url: Option<String>,

    /// Manifest `scope` [default: /].
    #[arg(long)]
    pub scope: Option<String>,

    /// Manifest `display` mode [default: standalone].
    #[arg(long, value_enum)]
    pub display: Option<Display>,

    /// Manifest icon `purpose` [default: "any maskable"].
    #[arg(long)]
    pub icon_purpose: Option<String>,

    /// `crossorigin` attribute for the manifest <link>, e.g. `use-credentials`.
    #[arg(long)]
    pub manifest_crossorigin: Option<String>,

    /// Figma personal access token (scope `file_content:read`), used when INPUT is a Figma link.
    #[arg(long, env = "FIGMA_TOKEN", hide_env_values = true)]
    pub figma_token: Option<String>,

    /// Read the Figma token from this file instead.
    #[arg(long, env = "FIGMA_TOKEN_FILE")]
    pub figma_token_file: Option<PathBuf>,

    /// Head snippets to write next to the icons [default: html,nuxt].
    #[arg(long, value_enum, value_delimiter = ',')]
    pub snippets: Option<Vec<Snippet>>,
}

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Display {
    Fullscreen,
    Standalone,
    MinimalUi,
    Browser,
}

impl Display {
    pub fn as_str(self) -> &'static str {
        match self {
            Display::Fullscreen => "fullscreen",
            Display::Standalone => "standalone",
            Display::MinimalUi => "minimal-ui",
            Display::Browser => "browser",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Snippet {
    /// favicon.html with <link>/<meta> tags.
    Html,
    /// nuxt-head.ts exporting `faviconHead` for `app.head`.
    Nuxt,
    /// Write no snippets.
    None,
}

pub fn parse_color(s: &str) -> Result<String, String> {
    rgb(s).map(|_| s.to_ascii_lowercase())
}

/// Parses `#rgb` / `#rrggbb`.
pub fn rgb(s: &str) -> Result<[u8; 3], String> {
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

pub fn is_figma_url(input: &str) -> bool {
    let rest = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .unwrap_or(input);
    rest.starts_with("figma.com/") || rest.starts_with("www.figma.com/")
}
