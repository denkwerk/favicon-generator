//! Reuses earlier results: the generated files, keyed by everything that
//! affects them, and Figma exports, which are checked against the file's
//! version before they are reused.
//!
//! Layout of the cache directory:
//! - `outputs/<key>/`: the generated files of one run
//! - `figma/<file key>/<node id>.svg` and `.json`: the last export of a node
//!   and the version of the Figma file it was exported from

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::Settings;
use crate::figma::NodeRef;
use crate::generate::OutputFile;

/// Output entries kept; older ones are removed when a new one is added.
const MAX_OUTPUTS: usize = 16;

/// Where the cache goes when no directory is configured.
pub const DEFAULT_DIR: &str = "node_modules/.cache/favicon-generator";

pub struct Cache {
    dir: PathBuf,
}

/// The input as read from disk or Figma, before it is decoded.
pub struct Input {
    pub bytes: Vec<u8>,
    pub is_svg: bool,
}

/// A cached Figma export.
pub struct FigmaExport {
    pub version: String,
    pub svg: Vec<u8>,
    /// When it was exported, for messages.
    pub exported: SystemTime,
}

#[derive(Deserialize, Serialize)]
struct FigmaMeta {
    version: String,
}

impl Cache {
    pub fn new(dir: PathBuf) -> Self {
        Cache { dir }
    }

    /// `node_modules/.cache/favicon-generator` in the project root (the
    /// nearest directory with a `package.json`), if it has a `node_modules`.
    pub fn default_dir(cwd: &Path) -> Option<PathBuf> {
        let root = cwd
            .ancestors()
            .find(|dir| dir.join("package.json").is_file())?;
        root.join("node_modules")
            .is_dir()
            .then(|| root.join(DEFAULT_DIR))
    }

    /// The files generated earlier for `key`, if any.
    pub fn outputs(&self, key: &str) -> Option<Vec<OutputFile>> {
        let dir = self.dir.join("outputs").join(key);
        let mut files = fs::read_dir(&dir)
            .ok()?
            .map(|entry| {
                let entry = entry.ok()?;
                Some(OutputFile {
                    name: entry.file_name().into_string().ok()?,
                    data: fs::read(entry.path()).ok()?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        files.sort_by(|a, b| a.name.cmp(&b.name));
        Some(files)
    }

    /// Stores the files for `key`, and removes the oldest entries beyond
    /// [`MAX_OUTPUTS`].
    pub fn store_outputs(&self, key: &str, files: &[OutputFile]) -> Result<()> {
        let outputs = self.dir.join("outputs");
        // Written next to the entry and renamed into place, so that a reader
        // never sees an incomplete entry.
        let tmp = outputs.join(format!(".tmp-{key}-{}", std::process::id()));
        fs::create_dir_all(&tmp).with_context(|| format!("failed to create {}", tmp.display()))?;
        for file in files {
            let path = tmp.join(&file.name);
            fs::write(&path, &file.data)
                .with_context(|| format!("failed to write {}", path.display()))?;
        }
        if fs::rename(&tmp, outputs.join(key)).is_err() {
            // Another run stored the same entry first.
            fs::remove_dir_all(&tmp).ok();
        }
        self.prune_outputs(&outputs);
        Ok(())
    }

    fn prune_outputs(&self, outputs: &Path) {
        let Ok(entries) = fs::read_dir(outputs) else {
            return;
        };
        let mut entries: Vec<(SystemTime, PathBuf)> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name();
                if name.to_string_lossy().starts_with('.') {
                    return None;
                }
                Some((entry.metadata().ok()?.modified().ok()?, entry.path()))
            })
            .collect();
        entries.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
        for (_, path) in entries.into_iter().skip(MAX_OUTPUTS) {
            fs::remove_dir_all(path).ok();
        }
    }

    fn figma_path(&self, node: &NodeRef, extension: &str) -> PathBuf {
        // `:` is not allowed in file names on Windows.
        self.dir
            .join("figma")
            .join(&node.file_key)
            .join(format!("{}.{extension}", node.node_id.replace(':', "-")))
    }

    /// The last export of `node`, if any.
    pub fn figma_export(&self, node: &NodeRef) -> Option<FigmaExport> {
        let meta_path = self.figma_path(node, "json");
        let meta: FigmaMeta = serde_json::from_slice(&fs::read(&meta_path).ok()?).ok()?;
        Some(FigmaExport {
            version: meta.version,
            svg: fs::read(self.figma_path(node, "svg")).ok()?,
            exported: fs::metadata(&meta_path).ok()?.modified().ok()?,
        })
    }

    pub fn store_figma_export(&self, node: &NodeRef, version: &str, svg: &[u8]) -> Result<()> {
        let svg_path = self.figma_path(node, "svg");
        let meta_path = self.figma_path(node, "json");
        let dir = svg_path.parent().expect("has a parent");
        fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
        // The version goes last: an interrupted write leaves an old version,
        // which only causes a new export.
        fs::remove_file(&meta_path).ok();
        fs::write(&svg_path, svg)
            .with_context(|| format!("failed to write {}", svg_path.display()))?;
        let meta = serde_json::to_vec(&FigmaMeta {
            version: version.to_owned(),
        })?;
        fs::write(&meta_path, meta)
            .with_context(|| format!("failed to write {}", meta_path.display()))?;
        Ok(())
    }
}

/// Hash of everything that affects the generated files: this version of the
/// generator, the settings and the input. `None` if the output also depends on
/// other files, i.e. an SVG that references external images.
pub fn key(settings: &Settings, input: &Input) -> Option<String> {
    if input.is_svg && references_external_images(&input.bytes) {
        return None;
    }
    // Destructured without `..`, so that a new setting cannot be forgotten here.
    let Settings {
        // The source is hashed by content, not by path.
        input: _,
        // Where and how the files are written does not change them.
        output: _,
        overwrite: _,
        cache: _,
        cache_dir: _,
        figma_token: _,
        figma_token_file: _,
        path_prefix,
        snippets,
        theme_color,
        apple_touch_icon,
        manifest,
        windows,
        legacy,
    } = settings;
    let settings = format!(
        "{path_prefix:?} {snippets:?} {theme_color:?} {apple_touch_icon:?} {manifest:?} {windows:?} {legacy:?}"
    );

    let mut hash = Sha256::new();
    for part in [
        env!("CARGO_PKG_VERSION").as_bytes(),
        settings.as_bytes(),
        if input.is_svg { b"svg" } else { b"raster" },
        &input.bytes,
    ] {
        // Length-prefixed, so that parts cannot run into each other.
        hash.update((part.len() as u64).to_le_bytes());
        hash.update(part);
    }
    Some(
        hash.finalize()[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

/// Whether an `<image>` in the SVG points at a file rather than a `data:` URL.
fn references_external_images(svg: &[u8]) -> bool {
    let svg = String::from_utf8_lossy(svg);
    svg.match_indices("<image").any(|(start, _)| {
        let tag = &svg[start..];
        let tag = &tag[..tag.find('>').unwrap_or(tag.len())];
        tag.match_indices("href=").any(|(at, attr)| {
            let value = tag[at + attr.len()..].trim_start_matches(['"', '\'']);
            !value.trim_start().starts_with("data:")
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_external_images() {
        assert!(!references_external_images(b"<svg><rect/></svg>"));
        assert!(!references_external_images(
            br#"<svg><image width="1" xlink:href="data:image/png;base64,AAAA"/></svg>"#
        ));
        assert!(references_external_images(
            br#"<svg><image href='photo.png'/></svg>"#
        ));
        assert!(references_external_images(
            br#"<svg><image href="data:x"/><image xlink:href="a.jpg"/></svg>"#
        ));
    }

    #[test]
    fn finds_the_default_dir_in_the_project_root() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("src/deep");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.path().join("package.json"), "{}").unwrap();
        assert_eq!(Cache::default_dir(&nested), None);

        std::fs::create_dir(root.path().join("node_modules")).unwrap();
        assert_eq!(
            Cache::default_dir(&nested),
            Some(root.path().join(DEFAULT_DIR))
        );
    }

    #[test]
    fn keeps_the_newest_outputs() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let files = [OutputFile {
            name: "a.txt".into(),
            data: b"a".to_vec(),
        }];
        for i in 0..MAX_OUTPUTS + 2 {
            cache.store_outputs(&format!("key{i:02}"), &files).unwrap();
            // Directory times are not always finer than this.
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            std::fs::read_dir(dir.path().join("outputs"))
                .unwrap()
                .count(),
            MAX_OUTPUTS
        );
        assert!(cache.outputs("key00").is_none());
        let last = cache
            .outputs(&format!("key{:02}", MAX_OUTPUTS + 1))
            .unwrap();
        assert_eq!(last[0].data, b"a");
    }
}
