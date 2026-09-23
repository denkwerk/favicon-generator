//! Helpers shared by the integration tests: a scratch project and a way to run
//! the compiled binary in it.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub const LOGO: &str = include_str!("../fixtures/logo.svg");

/// A temporary project directory with a `package.json` (so config discovery
/// stops here) and the fixture logo.
pub struct Project {
    pub dir: tempfile::TempDir,
}

impl Project {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"), "{}").unwrap();
        std::fs::write(dir.path().join("logo.svg"), LOGO).unwrap();
        Project { dir }
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.dir.path().join(relative)
    }

    pub fn write(&self, relative: &str, contents: impl AsRef<[u8]>) {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    pub fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.path(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_favicon-generator"));
        command
            .current_dir(self.dir.path())
            .env_remove("FIGMA_TOKEN")
            .env_remove("FIGMA_TOKEN_FILE");
        command
    }

    /// Runs the binary and asserts that it succeeds.
    pub fn run(&self, args: &[&str]) -> Output {
        let output = self.command().args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{args:?} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    /// Runs the binary, asserts that it fails, and returns its stderr.
    pub fn fail(&self, args: &[&str]) -> String {
        let output = self.command().args(args).output().unwrap();
        assert!(!output.status.success(), "{args:?} should have failed");
        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    /// Sorted file names in `dir`.
    pub fn files(&self, dir: &str) -> Vec<String> {
        list(&self.path(dir))
    }
}

pub fn list(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Whether Node.js is available to evaluate JS/TS configs.
pub fn has_node() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}
