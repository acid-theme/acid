//! What a port's test is given to work with.
//!
//! A test runs inside the container, where the port's own program is installed.
//! It reports each check it makes, and the run fails if any check fails or if
//! the program it needs is missing.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use acid_palette::Flavor;

pub struct Harness {
    /// The repository, mounted read-only inside the container.
    pub root: PathBuf,
    /// The port under test.
    pub port: String,
    passed: usize,
    failed: usize,
}

impl Harness {
    pub fn new(root: PathBuf, port: String) -> Self {
        Self {
            root,
            port,
            passed: 0,
            failed: 0,
        }
    }

    pub fn pass(&mut self, message: impl AsRef<str>) {
        self.passed += 1;
        println!("    ok   {}", message.as_ref());
    }

    pub fn fail(&mut self, message: impl AsRef<str>) {
        self.failed += 1;
        println!("    FAIL {}", message.as_ref());
    }

    pub fn note(&self, message: impl AsRef<str>) {
        println!("         {}", message.as_ref());
    }

    /// Assert a condition, reporting either way.
    pub fn check(&mut self, ok: bool, message: impl AsRef<str>) {
        if ok {
            self.pass(message);
        } else {
            self.fail(message);
        }
    }

    /// A check that looks for an error in a program's output passes when the
    /// program is missing, so its absence is a failure in itself.
    pub fn require(&mut self, programs: &[&str]) -> bool {
        for program in programs {
            if which(program).is_none() {
                self.fail(format!("{program} is not installed in the test image"));
                return false;
            }
        }
        true
    }

    /// Every flavour, so a test never names them.
    pub fn flavours(&self) -> &'static [Flavor] {
        &acid_palette::FLAVORS
    }

    /// A file this port publishes.
    pub fn dist(&self, relative: &str) -> PathBuf {
        self.root
            .join("ports")
            .join(&self.port)
            .join("dist")
            .join(relative)
    }

    pub fn failed(&self) -> bool {
        self.failed > 0
    }

    pub fn summary(&self) {
        println!("    -- {} passed, {} failed", self.passed, self.failed);
    }
}

/// Read a file, treating absence as empty.
pub fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Run a command, returning its combined output. Free of the harness so a test
/// can call it while reporting a result.
pub fn exec(program: &str, args: &[&str]) -> String {
    let Output { stdout, stderr, .. } = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("cannot run {program}: {error}"));
    format!(
        "{}{}",
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    )
}

/// Write a scratch file and return its path.
pub fn scratch(name: &str, contents: &str) -> std::io::Result<PathBuf> {
    let path = std::env::temp_dir().join(name);
    fs::write(&path, contents)?;
    Ok(path)
}

fn which(program: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(program))
            .find(|candidate| candidate.is_file())
    })
}
