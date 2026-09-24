use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

/// Generate favicons, an Apple touch icon and head tags from a single SVG
/// (recommended) or raster image, or from a Figma node.
///
/// Generates favicon.ico, favicon.svg, favicon-96x96.png, apple-touch-icon.png
/// and favicon.html. Additionally, the options below add a web app manifest, a
/// theme color, Windows tiles and legacy sizes.
///
/// Options can also be set in a favicon.config.{js,ts,mjs,mts,cjs,cts,json}
/// file, which is looked up in the current directory and its parents up to the
/// project root (the nearest directory with a package.json or .git).
/// Command-line flags take precedence over the config file.
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Same as --input.
    #[arg(value_name = "INPUT", conflicts_with = "input")]
    pub input_arg: Option<String>,

    /// Same as --output.
    #[arg(value_name = "OUTPUT", conflicts_with = "output")]
    pub output_arg: Option<PathBuf>,

    /// Source image (SVG, PNG, JPEG or WebP; should be square), or a Figma
    /// link with a `node-id` to export that node as SVG via the REST API.
    #[arg(short, long, help_heading = "Output")]
    pub input: Option<String>,

    /// Directory to write the generated files into [default: favicons].
    #[arg(short, long, help_heading = "Output")]
    pub output: Option<PathBuf>,

    /// URL prefix the files will be served from, e.g. `/favicons/` [default: /].
    #[arg(short = 'p', long, help_heading = "Output")]
    pub path_prefix: Option<String>,

    /// Overwrite existing files in the output directory.
    #[arg(short = 'y', long, help_heading = "Output")]
    pub overwrite: bool,

    /// Head snippets to write next to the icons [default: html].
    #[arg(long, value_enum, value_delimiter = ',', help_heading = "Output")]
    pub snippets: Option<Vec<Snippet>>,

    /// Config file to use instead of searching for one; `-` reads JSON from stdin.
    #[arg(short, long, conflicts_with = "no_config")]
    pub config: Option<PathBuf>,

    /// Ignore favicon.config.* files.
    #[arg(long)]
    pub no_config: bool,

    /// Print the config file that would be used, as JSON
    /// (`{ "path": ..., "config": ... }`, paths absolute), and exit.
    #[arg(long, conflicts_with = "no_config")]
    pub print_config: bool,

    /// Adds `<meta name="theme-color">` (and `theme_color` to the manifest).
    #[arg(long, value_parser = parse_color, help_heading = "Theme color")]
    pub theme_color: Option<String>,

    /// Generate apple-touch-icon.png (on by default).
    #[arg(
        long,
        overrides_with = "no_apple_touch_icon",
        help_heading = "Apple touch icon"
    )]
    pub apple_touch_icon: bool,

    /// Do not generate apple-touch-icon.png.
    #[arg(
        long,
        overrides_with = "apple_touch_icon",
        help_heading = "Apple touch icon"
    )]
    pub no_apple_touch_icon: bool,

    /// Color behind transparent pixels; iOS shows them black [default: #ffffff].
    #[arg(long, value_parser = parse_color, help_heading = "Apple touch icon")]
    pub apple_touch_background: Option<String>,

    /// Generate a web app manifest (implied by any manifest option).
    #[arg(
        long,
        overrides_with = "no_manifest",
        help_heading = "Web app manifest"
    )]
    pub manifest: bool,

    /// Do not generate a web app manifest, even if the config file enables it.
    #[arg(long, overrides_with = "manifest", help_heading = "Web app manifest")]
    pub no_manifest: bool,

    /// Manifest `name`, shown when installing the app.
    #[arg(short = 'n', long, help_heading = "Web app manifest")]
    pub name: Option<String>,

    /// Manifest `short_name`, shown on the home screen.
    #[arg(long, help_heading = "Web app manifest")]
    pub short_name: Option<String>,

    /// Manifest `description`.
    #[arg(long, help_heading = "Web app manifest")]
    pub description: Option<String>,

    /// Manifest `background_color` (splash screen); also behind maskable icons.
    #[arg(long, value_parser = parse_color, help_heading = "Web app manifest")]
    pub background_color: Option<String>,

    /// Manifest `start_url`.
    #[arg(long, help_heading = "Web app manifest")]
    pub start_url: Option<String>,

    /// Manifest `scope`.
    #[arg(long, help_heading = "Web app manifest")]
    pub scope: Option<String>,

    /// Manifest `display` mode.
    #[arg(long, value_enum, help_heading = "Web app manifest")]
    pub display: Option<Display>,

    /// Add maskable icons (the image at 60% on `background_color`) for Android.
    #[arg(long, help_heading = "Web app manifest")]
    pub maskable: bool,

    /// `crossorigin` attribute for the manifest `<link>`, e.g. `use-credentials`.
    #[arg(long, help_heading = "Web app manifest")]
    pub manifest_crossorigin: Option<String>,

    /// Generate browserconfig.xml and tile images (implied by --tile-color).
    #[arg(long, overrides_with = "no_windows", help_heading = "Windows tiles")]
    pub windows: bool,

    /// Do not generate Windows tiles, even if the config file enables them.
    #[arg(long, overrides_with = "windows", help_heading = "Windows tiles")]
    pub no_windows: bool,

    /// Tile color (`msapplication-TileColor`).
    #[arg(long, value_parser = parse_color, help_heading = "Windows tiles")]
    pub tile_color: Option<String>,

    /// Also generate the sizes old browsers and devices look for: 19 PNG
    /// sizes, sized Apple touch icons and a 7-frame favicon.ico.
    #[arg(long, overrides_with = "no_legacy", help_heading = "Legacy")]
    pub legacy: bool,

    /// Do not generate legacy sizes, even if the config file enables them.
    #[arg(long, overrides_with = "legacy", help_heading = "Legacy")]
    pub no_legacy: bool,

    /// Directory for the cache, which reuses the generated files while the
    /// input and the options are unchanged, and a Figma export while the Figma
    /// file is unchanged [default: node_modules/.cache/favicon-generator in the
    /// project root, if it has a node_modules directory].
    #[arg(
        long,
        env = "FAVICON_GENERATOR_CACHE_DIR",
        conflicts_with = "no_cache",
        help_heading = "Cache"
    )]
    pub cache_dir: Option<PathBuf>,

    /// Generate everything and export from Figma again, without reading or
    /// writing the cache.
    #[arg(long, help_heading = "Cache")]
    pub no_cache: bool,

    /// Figma personal access token (scope `file_content:read`), used when the input is a Figma link.
    #[arg(
        long,
        env = "FIGMA_TOKEN",
        hide_env_values = true,
        help_heading = "Figma"
    )]
    pub figma_token: Option<String>,

    /// Read the Figma token from this file instead.
    #[arg(long, env = "FIGMA_TOKEN_FILE", help_heading = "Figma")]
    pub figma_token_file: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum, Deserialize, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
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

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum, Deserialize, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Snippet {
    /// favicon.html with <link>/<meta> tags.
    Html,
    /// nuxt-head.ts exporting `faviconHead` for `app.head`.
    Nuxt,
    /// favicon-head.json with the tags as `{ link: [...], meta: [...] }`.
    Json,
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
