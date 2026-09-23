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

/// The contents of a config file. Every field mirrors a CLI flag in camelCase.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileConfig {
    #[serde(rename = "$schema", skip_serializing)]
    _schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_short_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tile_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_crossorigin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figma_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figma_token_file: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippets: Option<Vec<Snippet>>,
}

impl FileConfig {
    fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("invalid config")
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
pub struct Settings {
    pub input: String,
    pub output: PathBuf,
    pub overwrite: bool,
    pub path_prefix: String,
    pub app_name: String,
    pub app_short_name: String,
    pub app_description: String,
    pub theme_color: String,
    pub background_color: String,
    pub background_rgb: [u8; 3],
    pub tile_color: String,
    pub start_url: String,
    pub scope: String,
    pub display: Display,
    pub icon_purpose: String,
    pub manifest_crossorigin: Option<String>,
    pub figma_token: Option<String>,
    pub figma_token_file: Option<PathBuf>,
    pub snippets: Vec<Snippet>,
}

impl Settings {
    pub fn merge(cli: Cli, file: FileConfig) -> Result<Self> {
        let color =
            |cli: Option<String>, file: Option<String>, name: &str| -> Result<Option<String>> {
                match (cli, file) {
                    (Some(c), _) => Ok(Some(c)),
                    (None, Some(f)) => parse_color(&f)
                        .map(Some)
                        .map_err(|e| anyhow::anyhow!("invalid {name} in config: {e}")),
                    (None, None) => Ok(None),
                }
            };

        let Some(input) = cli.input.or(cli.input_arg).or(file.input) else {
            bail!("no input given: pass a file or Figma link, or set `input` in favicon.config.*");
        };
        let app_name = cli
            .app_name
            .or(file.app_name)
            .unwrap_or_else(|| "App".into());
        let background_color = color(
            cli.background_color,
            file.background_color,
            "backgroundColor",
        )?
        .unwrap_or_else(|| "#ffffff".into());
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
            .unwrap_or(vec![Snippet::Html, Snippet::Nuxt]);
        let snippets = if snippets.contains(&Snippet::None) {
            Vec::new()
        } else {
            snippets
        };

        Ok(Settings {
            input,
            output: cli
                .output
                .or(cli.output_arg)
                .or(file.output)
                .unwrap_or_else(|| "favicons".into()),
            overwrite: cli.overwrite || file.overwrite.unwrap_or(false),
            path_prefix,
            app_short_name: cli
                .app_short_name
                .or(file.app_short_name)
                .unwrap_or_else(|| app_name.clone()),
            app_description: cli
                .app_description
                .or(file.app_description)
                .unwrap_or_else(|| app_name.clone()),
            app_name,
            theme_color: color(cli.theme_color, file.theme_color, "themeColor")?
                .unwrap_or_else(|| "#ffffff".into()),
            tile_color: color(cli.tile_color, file.tile_color, "tileColor")?
                .unwrap_or_else(|| background_color.clone()),
            background_rgb: rgb(&background_color).expect("validated above"),
            background_color,
            start_url: cli
                .start_url
                .or(file.start_url)
                .unwrap_or_else(|| "/?source=pwa".into()),
            scope: cli.scope.or(file.scope).unwrap_or_else(|| "/".into()),
            display: cli.display.or(file.display).unwrap_or(Display::Standalone),
            icon_purpose: cli
                .icon_purpose
                .or(file.icon_purpose)
                .unwrap_or_else(|| "any maskable".into()),
            manifest_crossorigin: cli.manifest_crossorigin.or(file.manifest_crossorigin),
            figma_token: cli.figma_token.or(file.figma_token),
            figma_token_file: cli.figma_token_file.or(file.figma_token_file),
            snippets,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

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
            r##"{ "input": "logo.svg", "output": "public", "display": "minimal-ui", "snippets": ["html"] }"##,
        )
        .unwrap();
        let config = load(&path).unwrap();
        assert_eq!(
            config.input.unwrap(),
            dir.path().join("logo.svg").to_string_lossy()
        );
        assert_eq!(config.output.unwrap(), dir.path().join("public"));
        assert_eq!(config.display, Some(Display::MinimalUi));
        assert_eq!(config.snippets, Some(vec![Snippet::Html]));
    }

    #[test]
    fn rejects_unknown_keys() {
        let err = FileConfig::from_json(r#"{ "apName": "x" }"#).unwrap_err();
        assert!(format!("{err:#}").contains("unknown field `apName`"));
    }

    #[test]
    fn cli_overrides_config() {
        let cli = Cli::parse_from([
            "favicon-generator",
            "--app-name",
            "Cli",
            "--theme-color",
            "#000",
        ]);
        let file = FileConfig::from_json(
            r##"{ "input": "https://www.figma.com/design/K/x?node-id=1-2", "appName": "File", "themeColor": "#FFF", "backgroundColor": "#123", "snippets": [] }"##,
        )
        .unwrap();
        let s = Settings::merge(cli, file).unwrap();
        assert_eq!(s.input, "https://www.figma.com/design/K/x?node-id=1-2");
        assert_eq!(
            (s.app_name.as_str(), s.app_short_name.as_str()),
            ("Cli", "Cli")
        );
        assert_eq!(s.theme_color, "#000");
        assert_eq!(s.tile_color, "#123");
        assert!(s.snippets.is_empty());
    }

    #[test]
    fn input_and_output_as_flags_or_arguments() {
        let empty = || FileConfig::from_json("{}").unwrap();
        for args in [
            &["favicon-generator", "-i", "logo.svg", "-o", "public"][..],
            &[
                "favicon-generator",
                "--input",
                "logo.svg",
                "--output",
                "public",
            ],
            &["favicon-generator", "logo.svg", "public"],
        ] {
            let s = Settings::merge(Cli::parse_from(args), empty()).unwrap();
            assert_eq!(
                (s.input.as_str(), s.output.as_path()),
                ("logo.svg", Path::new("public"))
            );
        }

        let file =
            FileConfig::from_json(r#"{ "input": "file.svg", "output": "file-out" }"#).unwrap();
        let s = Settings::merge(
            Cli::parse_from(["favicon-generator", "-o", "cli-out"]),
            file,
        )
        .unwrap();
        assert_eq!(
            (s.input.as_str(), s.output.as_path()),
            ("file.svg", Path::new("cli-out"))
        );

        assert!(Cli::try_parse_from(["favicon-generator", "a.svg", "--input", "b.svg"]).is_err());
        assert!(Cli::try_parse_from(["favicon-generator", "a.svg", "out", "-o", "other"]).is_err());
    }

    #[test]
    fn validates_config_colors() {
        let cli = Cli::parse_from(["favicon-generator", "logo.svg"]);
        let file = FileConfig::from_json(r#"{ "themeColor": "white" }"#).unwrap();
        assert!(Settings::merge(cli, file).is_err());
    }
}
