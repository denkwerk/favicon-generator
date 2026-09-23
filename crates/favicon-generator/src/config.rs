//! `favicon.config.*` discovery and loading, and merging with CLI flags.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::cli::{Cli, Display, Snippet, is_figma_url, parse_color, rgb};

/// Looked up in this order; the first match in a directory wins.
pub const FILE_NAMES: [&str; 7] = [
    "favicon.config.js",
    "favicon.config.ts",
    "favicon.config.mjs",
    "favicon.config.mts",
    "favicon.config.cjs",
    "favicon.config.cts",
    "favicon.config.json",
];

/// Node.js binary used to evaluate JS/TS configs. The npm wrapper sets this to
/// the Node.js that launched it.
const NODE_ENV: &str = "FAVICON_GENERATOR_NODE";

const MARKER: &str = "__FAVICON_GENERATOR_CONFIG__";

/// Imports the config module and prints its default export as JSON.
const LOADER: &str = r#"
const { pathToFileURL } = await import('node:url');
const file = process.argv[1];
let mod;
try {
  mod = await import(pathToFileURL(file).href);
} catch (error) {
  if (error?.code === 'ERR_UNKNOWN_FILE_EXTENSION' && /\.[cm]?ts$/.test(file)) {
    console.error(`Node.js ${process.version} cannot load TypeScript config files; use Node.js >= 22.18, or a .js/.mjs config.`);
    process.exit(3);
  }
  throw error;
}
let config = mod.default;
if (typeof config === 'function') config = await config();
if (config === null || typeof config !== 'object' || Array.isArray(config)) {
  console.error(`${file} must export a config object as its default export (see defineConfig).`);
  process.exit(3);
}
process.stdout.write('\n' + MARKER + JSON.stringify(config) + '\n');
"#;

/// The contents of a config file. Top-level fields mirror the output flags;
/// optional outputs are grouped, and each group can be `true` (defaults),
/// `false` (off) or an object with its options (on).
#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, schemars(title = "favicon-generator config"))]
pub struct FileConfig {
    /// Path or URL of this JSON Schema, for editor support.
    #[serde(rename = "$schema", skip_serializing)]
    pub schema: Option<String>,
    /// Source image (SVG, PNG, JPEG or WebP; should be square), or a Figma link
    /// with a `node-id`, which is exported as SVG through the Figma REST API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    /// Directory to write the generated files into. Default: `favicons`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<PathBuf>,
    /// Overwrite existing files in the output directory. Default: `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<bool>,
    /// URL prefix the files will be served from, e.g. `/favicons/`. Default: `/`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
    /// Head snippets to write next to the icons; `[]` writes none. Default: `["html"]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippets: Option<Vec<Snippet>>,
    /// Adds `<meta name="theme-color">` (and `theme_color` to the manifest).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, schemars(schema_with = "color_schema"))]
    pub theme_color: Option<String>,
    /// `apple-touch-icon.png`, 180×180 and opaque. On by default; `false` turns it off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apple_touch_icon: Option<Toggle<AppleTouchIconConfig>>,
    /// A web app manifest (`manifest.json`) with 192 and 512 px icons. Off unless configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest: Option<Toggle<ManifestConfig>>,
    /// `browserconfig.xml` and tile images for pinned sites on Windows. Off unless configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<Toggle<WindowsConfig>>,
    /// Also generate the sizes old browsers and devices look for: 19 PNG sizes,
    /// sized Apple touch icons and a 7-frame `favicon.ico`. Default: `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy: Option<bool>,
    /// Figma personal access token (scope `file_content:read`). Prefer the
    /// `FIGMA_TOKEN` env var or `figmaTokenFile` over committing a token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figma_token: Option<String>,
    /// File containing the Figma personal access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figma_token_file: Option<PathBuf>,
}

/// Options of the Apple touch icon.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppleTouchIconConfig {
    /// Color behind transparent pixels; iOS shows them black. Default: `#ffffff`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, schemars(schema_with = "color_schema"))]
    pub background: Option<String>,
}

/// Options of the web app manifest. Only the fields you set are written.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestConfig {
    /// `name`, shown when installing the app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// `short_name`, shown on the home screen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// `description`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// `background_color` of the splash screen; also behind maskable icons.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, schemars(schema_with = "color_schema"))]
    pub background_color: Option<String>,
    /// `start_url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_url: Option<String>,
    /// `scope`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// `display` mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
    /// Add maskable icons (the image at 60% on `backgroundColor`) for Android. Default: `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maskable: Option<bool>,
    /// `crossorigin` attribute for the manifest `<link>`, e.g. `use-credentials`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossorigin: Option<String>,
}

/// Options of the Windows tiles.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowsConfig {
    /// Tile color (`msapplication-TileColor`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, schemars(schema_with = "color_schema"))]
    pub tile_color: Option<String>,
}

/// An optional output: `true` (defaults), `false` (off) or its options (on).
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Toggle<T> {
    Enabled(bool),
    Options(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Toggle<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
            type Value = Toggle<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("true, false or an options object")
            }
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(Toggle::Enabled(value))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                // Deserializing the options directly keeps serde's messages for unknown keys.
                T::deserialize(serde::de::value::MapAccessDeserializer::new(map))
                    .map(Toggle::Options)
            }
        }
        deserializer.deserialize_any(Visitor(std::marker::PhantomData))
    }
}

#[cfg(test)]
impl<T: schemars::JsonSchema> schemars::JsonSchema for Toggle<T> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        format!("Toggle_{}", T::schema_name()).into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "anyOf": [{ "type": "boolean" }, generator.subschema_for::<T>()]
        })
    }
}

#[cfg(test)]
fn color_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "type": "string",
        "pattern": "^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$",
        "description": "`#rgb` or `#rrggbb`"
    })
}

/// Options that 0.2.0 moved into groups, with their new place.
const RENAMED: [(&str, &str); 10] = [
    ("appName", "manifest.name"),
    ("appShortName", "manifest.shortName"),
    ("appDescription", "manifest.description"),
    (
        "backgroundColor",
        "manifest.backgroundColor or appleTouchIcon.background",
    ),
    ("tileColor", "windows.tileColor"),
    ("startUrl", "manifest.startUrl"),
    ("scope", "manifest.scope"),
    ("display", "manifest.display"),
    ("iconPurpose", "manifest.maskable"),
    ("manifestCrossorigin", "manifest.crossorigin"),
];

impl FileConfig {
    pub fn from_json(json: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(json).context("invalid config")?;
        if let Some(object) = value.as_object() {
            let renamed: Vec<String> = RENAMED
                .iter()
                .filter(|(old, _)| object.contains_key(*old))
                .map(|(old, new)| format!("`{old}` is now `{new}`"))
                .collect();
            if !renamed.is_empty() {
                bail!(
                    "invalid config: {} (options were regrouped in 0.2.0)",
                    renamed.join(", ")
                );
            }
        }
        serde_json::from_value(value).context("invalid config")
    }

    /// Makes relative paths relative to `base` (the config file's directory).
    fn resolve_paths(mut self, base: &Path) -> Self {
        if let Some(input) = &self.input
            && !is_figma_url(input)
        {
            self.input = Some(join(base, input).to_string_lossy().into_owned());
        }
        self.output = self.output.map(|p| join(base, p));
        self.figma_token_file = self.figma_token_file.map(|p| join(base, p));
        self
    }
}

/// `base.join(path)` without `.` components, for readable messages.
fn join(base: &Path, path: impl AsRef<Path>) -> PathBuf {
    base.join(path)
        .components()
        .filter(|c| !matches!(c, std::path::Component::CurDir))
        .collect()
}

/// Finds a config file in `start` or its ancestors, stopping at the project
/// root: the first directory that contains a `package.json` or `.git`.
pub fn discover(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let mut found = FILE_NAMES
            .iter()
            .map(|name| dir.join(name))
            .filter(|p| p.is_file());
        if let Some(first) = found.next() {
            if let Some(other) = found.next() {
                eprintln!(
                    "warning: found both {} and {}; using the former",
                    first.display(),
                    other.display()
                );
            }
            return Some(first);
        }
        if dir.join("package.json").is_file() || dir.join(".git").exists() {
            break;
        }
    }
    None
}

/// Loads `path` (JSON natively, JS/TS through Node.js), with paths resolved
/// against its directory.
pub fn load(path: &Path) -> Result<FileConfig> {
    let is_json = path.extension().is_some_and(|e| e == "json");
    let json = if is_json {
        std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?
    } else {
        evaluate_with_node(path)?
    };
    let config = FileConfig::from_json(&json).with_context(|| format!("in {}", path.display()))?;
    let base = path.parent().unwrap_or(Path::new("."));
    Ok(config.resolve_paths(base))
}

/// Reads a JSON config from stdin; paths are resolved against `cwd`.
pub fn load_stdin(cwd: &Path) -> Result<FileConfig> {
    let mut json = String::new();
    std::io::stdin()
        .read_to_string(&mut json)
        .context("failed to read config from stdin")?;
    Ok(FileConfig::from_json(&json)
        .context("in config from stdin")?
        .resolve_paths(cwd))
}

fn evaluate_with_node(path: &Path) -> Result<String> {
    let node = std::env::var_os(NODE_ENV).unwrap_or_else(|| "node".into());
    let loader = LOADER.replace("MARKER", &format!("'{MARKER}'"));
    let output = Command::new(&node)
        .args([
            "--input-type=module",
            "--disable-warning=ExperimentalWarning",
            "--disable-warning=MODULE_TYPELESS_PACKAGE_JSON",
            "-e",
        ])
        .arg(loader)
        .arg("--")
        .arg(path)
        .current_dir(path.parent().unwrap_or(Path::new(".")))
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output();
    let output = match output {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => bail!(
            "Node.js is required to load {} but `{}` was not found; install Node.js or use favicon.config.json",
            path.display(),
            node.to_string_lossy()
        ),
        Err(err) => return Err(err).context("failed to start Node.js"),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut json = None;
    for line in stdout.lines() {
        match line.strip_prefix(MARKER) {
            Some(rest) => json = Some(rest.to_owned()),
            // Forward anything the config itself printed.
            None if !line.is_empty() => eprintln!("{line}"),
            None => {}
        }
    }
    match json {
        Some(json) if output.status.success() => Ok(json),
        _ => bail!("failed to evaluate {} with Node.js", path.display()),
    }
}

/// Fully resolved options: CLI flags over config file over defaults.
#[derive(Debug)]
pub struct Settings {
    pub input: String,
    pub output: PathBuf,
    pub overwrite: bool,
    pub path_prefix: String,
    pub snippets: Vec<Snippet>,
    pub theme_color: Option<String>,
    pub apple_touch_icon: Option<AppleTouchIcon>,
    pub manifest: Option<Manifest>,
    pub windows: Option<Windows>,
    pub legacy: bool,
    pub figma_token: Option<String>,
    pub figma_token_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct AppleTouchIcon {
    pub background: [u8; 3],
}

#[derive(Debug, Default)]
pub struct Manifest {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub background_color: Option<String>,
    pub start_url: Option<String>,
    pub scope: Option<String>,
    pub display: Option<Display>,
    pub maskable: bool,
    pub crossorigin: Option<String>,
}

#[derive(Debug)]
pub struct Windows {
    pub tile_color: Option<String>,
}

/// Whether an optional output is on, and the file's options for it. Flags win:
/// `--no-<group>` turns it off, `--<group>` or any of its options turn it on.
fn group<T: Default>(
    file: Option<Toggle<T>>,
    on: bool,
    off: bool,
    flags_set: bool,
    default: bool,
) -> Option<T> {
    let enabled = if off {
        false
    } else if on || flags_set {
        true
    } else {
        match &file {
            None => default,
            Some(Toggle::Enabled(enabled)) => *enabled,
            Some(Toggle::Options(_)) => true,
        }
    };
    enabled.then(|| match file {
        Some(Toggle::Options(options)) => options,
        _ => T::default(),
    })
}

/// A color from the CLI (already validated) or the config file (validated here).
fn color(cli: Option<String>, file: Option<String>, name: &str) -> Result<Option<String>> {
    match (cli, file) {
        (Some(color), _) => Ok(Some(color)),
        (None, Some(color)) => parse_color(&color)
            .map(Some)
            .map_err(|e| anyhow::anyhow!("invalid {name} in config: {e}")),
        (None, None) => Ok(None),
    }
}

impl Settings {
    pub fn merge(cli: Cli, file: FileConfig) -> Result<Self> {
        let Some(input) = cli.input.or(cli.input_arg).or(file.input) else {
            bail!(
                "no input given: pass -i <file or Figma link>, or set `input` in favicon.config.*"
            );
        };
        let mut path_prefix = cli
            .path_prefix
            .or(file.path_prefix)
            .unwrap_or_else(|| "/".into());
        if !path_prefix.ends_with('/') {
            path_prefix.push('/');
        }
        let snippets = cli
            .snippets
            .or(file.snippets)
            .unwrap_or(vec![Snippet::Html]);
        let snippets = if snippets.contains(&Snippet::None) {
            Vec::new()
        } else {
            snippets
        };

        let apple_touch_icon = match group(
            file.apple_touch_icon,
            cli.apple_touch_icon,
            cli.no_apple_touch_icon,
            cli.apple_touch_background.is_some(),
            true,
        ) {
            Some(options) => {
                let background = color(
                    cli.apple_touch_background,
                    options.background,
                    "appleTouchIcon.background",
                )?
                .unwrap_or_else(|| "#ffffff".into());
                Some(AppleTouchIcon {
                    background: rgb(&background).expect("validated color"),
                })
            }
            None => None,
        };

        let manifest_flags = cli.name.is_some()
            || cli.short_name.is_some()
            || cli.description.is_some()
            || cli.background_color.is_some()
            || cli.start_url.is_some()
            || cli.scope.is_some()
            || cli.display.is_some()
            || cli.maskable
            || cli.manifest_crossorigin.is_some();
        let manifest = match group(
            file.manifest,
            cli.manifest,
            cli.no_manifest,
            manifest_flags,
            false,
        ) {
            Some(options) => Some(Manifest {
                name: cli.name.or(options.name),
                short_name: cli.short_name.or(options.short_name),
                description: cli.description.or(options.description),
                background_color: color(
                    cli.background_color,
                    options.background_color,
                    "manifest.backgroundColor",
                )?,
                start_url: cli.start_url.or(options.start_url),
                scope: cli.scope.or(options.scope),
                display: cli.display.or(options.display),
                maskable: cli.maskable || options.maskable.unwrap_or(false),
                crossorigin: cli.manifest_crossorigin.or(options.crossorigin),
            }),
            None => None,
        };

        let windows = match group(
            file.windows,
            cli.windows,
            cli.no_windows,
            cli.tile_color.is_some(),
            false,
        ) {
            Some(options) => Some(Windows {
                tile_color: color(cli.tile_color, options.tile_color, "windows.tileColor")?,
            }),
            None => None,
        };

        let legacy = !cli.no_legacy && (cli.legacy || file.legacy.unwrap_or(false));

        Ok(Settings {
            input,
            output: cli
                .output
                .or(cli.output_arg)
                .or(file.output)
                .unwrap_or_else(|| "favicons".into()),
            overwrite: cli.overwrite || file.overwrite.unwrap_or(false),
            path_prefix,
            snippets,
            theme_color: color(cli.theme_color, file.theme_color, "themeColor")?,
            apple_touch_icon,
            manifest,
            windows,
            legacy,
            figma_token: cli.figma_token.or(file.figma_token),
            figma_token_file: cli.figma_token_file.or(file.figma_token_file),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn cli(args: &[&str]) -> Cli {
        Cli::parse_from(std::iter::once("favicon-generator").chain(args.iter().copied()))
    }

    fn file(json: &str) -> FileConfig {
        FileConfig::from_json(json).unwrap()
    }

    fn merge(args: &[&str], json: &str) -> Settings {
        Settings::merge(cli(args), file(json)).unwrap()
    }

    #[test]
    fn discovers_up_to_the_project_root() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("project");
        let nested = project.join("src/deep");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(project.join("package.json"), "{}").unwrap();

        // Nothing found, and the search does not escape the project root.
        std::fs::write(root.path().join("favicon.config.json"), "{}").unwrap();
        assert_eq!(discover(&nested), None);

        std::fs::write(project.join("favicon.config.ts"), "").unwrap();
        std::fs::write(project.join("favicon.config.json"), "").unwrap();
        assert_eq!(discover(&nested), Some(project.join("favicon.config.ts")));
    }

    #[test]
    fn loads_json_relative_to_its_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("favicon.config.json");
        std::fs::write(
            &path,
            r#"{ "input": "logo.svg", "output": "public", "figmaTokenFile": "token", "manifest": { "display": "minimal-ui" } }"#,
        )
        .unwrap();
        let config = load(&path).unwrap();
        assert_eq!(
            config.input.unwrap(),
            dir.path().join("logo.svg").to_string_lossy()
        );
        assert_eq!(config.output.unwrap(), dir.path().join("public"));
        assert_eq!(config.figma_token_file.unwrap(), dir.path().join("token"));
        let Some(Toggle::Options(manifest)) = config.manifest else {
            panic!("manifest options")
        };
        assert_eq!(manifest.display, Some(Display::MinimalUi));
    }

    #[test]
    fn keeps_figma_links_as_they_are() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("favicon.config.json");
        let link = "https://www.figma.com/design/K/x?node-id=1-2";
        std::fs::write(&path, format!(r#"{{ "input": "{link}" }}"#)).unwrap();
        assert_eq!(load(&path).unwrap().input.unwrap(), link);
    }

    #[test]
    fn rejects_unknown_keys() {
        let err = FileConfig::from_json(r##"{ "themeColr": "#fff" }"##).unwrap_err();
        assert!(format!("{err:#}").contains("unknown field `themeColr`"));
        let err = FileConfig::from_json(r#"{ "manifest": { "nme": "x" } }"#).unwrap_err();
        assert!(
            format!("{err:#}").contains("unknown field `nme`"),
            "{err:#}"
        );
    }

    #[test]
    fn names_the_replacement_of_renamed_options() {
        let err =
            FileConfig::from_json(r##"{ "appName": "x", "tileColor": "#fff" }"##).unwrap_err();
        let message = format!("{err:#}");
        assert!(
            message.contains("`appName` is now `manifest.name`"),
            "{message}"
        );
        assert!(
            message.contains("`tileColor` is now `windows.tileColor`"),
            "{message}"
        );
    }

    #[test]
    fn groups_accept_booleans_and_objects() {
        let config = file(
            r##"{ "manifest": true, "windows": false, "appleTouchIcon": { "background": "#000" } }"##,
        );
        assert!(matches!(config.manifest, Some(Toggle::Enabled(true))));
        assert!(matches!(config.windows, Some(Toggle::Enabled(false))));
        assert!(matches!(config.apple_touch_icon, Some(Toggle::Options(_))));
        let err = FileConfig::from_json(r#"{ "manifest": "yes" }"#).unwrap_err();
        assert!(
            format!("{err:#}").contains("true, false or an options object"),
            "{err:#}"
        );
    }

    #[test]
    fn defaults_are_minimal() {
        let s = merge(&["-i", "logo.svg"], "{}");
        assert_eq!(s.output, Path::new("favicons"));
        assert_eq!(s.path_prefix, "/");
        assert_eq!(s.snippets, vec![Snippet::Html]);
        assert_eq!(s.theme_color, None);
        assert_eq!(s.apple_touch_icon.unwrap().background, [255, 255, 255]);
        assert!(s.manifest.is_none());
        assert!(s.windows.is_none());
        assert!(!s.legacy);
    }

    #[test]
    fn input_and_output_as_flags_or_arguments() {
        for args in [
            &["-i", "logo.svg", "-o", "public"][..],
            &["--input", "logo.svg", "--output", "public"],
            &["logo.svg", "public"],
        ] {
            let s = merge(args, "{}");
            assert_eq!(
                (s.input.as_str(), s.output.as_path()),
                ("logo.svg", Path::new("public"))
            );
        }
        let s = merge(
            &["-o", "cli-out"],
            r#"{ "input": "file.svg", "output": "file-out" }"#,
        );
        assert_eq!(
            (s.input.as_str(), s.output.as_path()),
            ("file.svg", Path::new("cli-out"))
        );

        assert!(Cli::try_parse_from(["favicon-generator", "a.svg", "--input", "b.svg"]).is_err());
        assert!(Cli::try_parse_from(["favicon-generator", "a.svg", "out", "-o", "other"]).is_err());
        assert!(Settings::merge(cli(&[]), file("{}")).is_err());
    }

    #[test]
    fn a_manifest_option_turns_the_manifest_on() {
        let s = merge(&["-i", "a.svg", "--name", "My App"], "{}");
        let manifest = s.manifest.unwrap();
        assert_eq!(manifest.name.as_deref(), Some("My App"));
        assert_eq!(manifest.display, None);

        let s = merge(&["-i", "a.svg", "--manifest"], "{}");
        assert!(s.manifest.is_some_and(|m| m.name.is_none()));

        let s = merge(
            &["-i", "a.svg"],
            r#"{ "manifest": { "shortName": "App" } }"#,
        );
        assert_eq!(s.manifest.unwrap().short_name.as_deref(), Some("App"));
    }

    #[test]
    fn flags_override_the_config_file_per_option() {
        let json = r##"{
            "input": "a.svg",
            "themeColor": "#111111",
            "manifest": { "name": "File", "shortName": "F", "display": "browser" },
            "windows": { "tileColor": "#222222" },
            "legacy": true
        }"##;
        let s = merge(&["--name", "Cli", "--theme-color", "#000"], json);
        let manifest = s.manifest.unwrap();
        assert_eq!(manifest.name.as_deref(), Some("Cli"));
        assert_eq!(manifest.short_name.as_deref(), Some("F"));
        assert_eq!(manifest.display, Some(Display::Browser));
        assert_eq!(s.theme_color.as_deref(), Some("#000"));
        assert_eq!(s.windows.unwrap().tile_color.as_deref(), Some("#222222"));
        assert!(s.legacy);

        let s = merge(
            &[
                "--no-manifest",
                "--no-windows",
                "--no-legacy",
                "--no-apple-touch-icon",
            ],
            json,
        );
        assert!(
            s.manifest.is_none()
                && s.windows.is_none()
                && !s.legacy
                && s.apple_touch_icon.is_none()
        );
    }

    #[test]
    fn false_turns_a_group_off_unless_a_flag_turns_it_on() {
        let json = r#"{ "input": "a.svg", "appleTouchIcon": false, "manifest": false }"#;
        let s = merge(&[], json);
        assert!(s.apple_touch_icon.is_none() && s.manifest.is_none());
        let s = merge(&["--apple-touch-icon", "--name", "App"], json);
        assert!(s.apple_touch_icon.is_some() && s.manifest.is_some());
    }

    #[test]
    fn the_last_of_a_flag_pair_wins() {
        assert!(
            merge(&["-i", "a.svg", "--manifest", "--no-manifest"], "{}")
                .manifest
                .is_none()
        );
        assert!(
            merge(&["-i", "a.svg", "--no-manifest", "--manifest"], "{}")
                .manifest
                .is_some()
        );
    }

    #[test]
    fn validates_colors() {
        assert!(Settings::merge(cli(&["a.svg"]), file(r#"{ "themeColor": "white" }"#)).is_err());
        let err = Settings::merge(
            cli(&["a.svg"]),
            file(r##"{ "manifest": { "backgroundColor": "#12" } }"##),
        )
        .unwrap_err();
        assert!(
            format!("{err:#}").contains("manifest.backgroundColor"),
            "{err:#}"
        );
        assert!(Cli::try_parse_from(["favicon-generator", "--theme-color", "red"]).is_err());
        let s = merge(&["-i", "a.svg", "--apple-touch-background", "#ABC"], "{}");
        assert_eq!(s.apple_touch_icon.unwrap().background, [0xaa, 0xbb, 0xcc]);
    }

    #[test]
    fn snippets_none_writes_no_snippets() {
        assert!(
            merge(&["-i", "a.svg", "--snippets", "none"], "{}")
                .snippets
                .is_empty()
        );
        assert!(
            merge(&["-i", "a.svg"], r#"{ "snippets": [] }"#)
                .snippets
                .is_empty()
        );
    }

    /// The JSON Schema shipped with the npm package, generated from these types.
    /// Run with `UPDATE_SCHEMA=1` to rewrite it.
    #[test]
    fn schema_is_up_to_date() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/favicon-generator/schema.json");
        let schema = schemars::generate::SchemaSettings::draft07()
            .into_generator()
            .into_root_schema_for::<FileConfig>();
        let json = serde_json::to_string_pretty(&schema).unwrap() + "\n";
        if std::env::var_os("UPDATE_SCHEMA").is_some() {
            std::fs::write(&path, &json).unwrap();
        }
        let committed = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .replace("\r\n", "\n");
        assert!(
            committed == json,
            "{} is outdated; run `UPDATE_SCHEMA=1 cargo test`",
            path.display()
        );
    }
}
