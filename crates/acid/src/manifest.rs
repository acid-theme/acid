//! What a port is, and where the project publishes it.
//!
//! A port is a directory under `ports/` holding everything about it: the
//! manifest, the template, its README, its test and its preview. There is no
//! central registry — the directory listing is the registry.

// Some of this is read by commands not yet written, and some by people.
#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

/// `acid.toml` at the repository root.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The organisation the ports are published to.
    pub org: String,
    /// The hub repository, as `owner/name`.
    pub hub: String,
    /// The hub ref a port's preview workflow calls.
    #[serde(default = "default_ref")]
    pub hub_ref: String,
}

fn default_ref() -> String {
    "main".to_owned()
}

/// `ports/<name>/port.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// The port's display name, e.g. `Telegram Desktop`.
    pub title: String,
    /// One line, used as the repository description.
    pub description: String,
    /// The repository name, when it differs from the directory name.
    #[serde(default)]
    pub repo: Option<String>,
    /// The packages this port's test needs. Absent when it needs none.
    #[serde(default)]
    pub test: Option<Packages>,
    /// Absent when the program cannot be rendered headlessly.
    #[serde(default)]
    pub preview: Option<Packages>,
    /// How the theme is rendered.
    pub render: Render,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Render {
    /// The axes to render across. `flavor` plus at most one of the port's own.
    pub matrix: Vec<Axis>,
    /// Where the rendered file goes, relative to the port's directory.
    pub filename: PerVariant,
    /// A string the template can use, which may differ per variant.
    #[serde(default)]
    pub prefix: Option<PerVariant>,
}

/// One axis of the render matrix: `"flavor"`, or a name with its values.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Axis {
    Flavor(String),
    Custom(std::collections::BTreeMap<String, Vec<String>>),
}

/// A value that is either the same for every variant or given per variant.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PerVariant {
    Same(String),
    PerVariant(std::collections::BTreeMap<String, String>),
}

impl PerVariant {
    /// The value for a variant, falling back to the shared one.
    pub fn get(&self, variant: &str) -> &str {
        match self {
            Self::Same(value) => value,
            Self::PerVariant(map) => map.get(variant).map_or("", String::as_str),
        }
    }
}

/// The packages a container needs, beyond the shared base.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Packages {
    pub packages: Vec<String>,
}

#[derive(Debug)]
pub struct Port {
    /// The directory name, which is also the port's identifier.
    pub name: String,
    pub dir: PathBuf,
    pub manifest: Manifest,
}

impl Port {
    /// The repository this port publishes to, as `owner/name`.
    pub fn slug(&self, config: &Config) -> String {
        let repo = self.manifest.repo.as_deref().unwrap_or(&self.name);
        format!("{}/{repo}", config.org)
    }

    pub fn template(&self) -> PathBuf {
        self.dir.join("theme.tera")
    }

    pub fn readme(&self) -> PathBuf {
        self.dir.join("README.tera")
    }

    /// What publishing copies: laid out exactly as the port's repository.
    pub fn dist(&self) -> PathBuf {
        self.dir.join("dist")
    }

    pub fn test(&self) -> PathBuf {
        self.dir.join("test.rs")
    }

    pub fn preview_script(&self) -> PathBuf {
        self.dir.join("preview.rs")
    }
}

pub fn root() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/acid; the repository is two up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn config(root: &Path) -> Result<Config> {
    let path = root.join("acid.toml");
    let text =
        fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("{} is not valid", path.display()))
}

/// Every port, in directory order, which is the order everything presents them.
pub fn ports(root: &Path) -> Result<Vec<Port>> {
    let mut found = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(root.join("ports"))
        .context("cannot read ports/")?
        .collect::<Result<_, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let manifest_path = dir.join("port.toml");
        if !manifest_path.is_file() {
            bail!("{} has no port.toml", dir.display());
        }
        let text = fs::read_to_string(&manifest_path)?;
        let manifest: Manifest = toml::from_str(&text)
            .with_context(|| format!("{} is not valid", manifest_path.display()))?;
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .context("a port directory has no name")?
            .to_owned();
        found.push(Port {
            name,
            dir,
            manifest,
        });
    }

    if found.is_empty() {
        bail!("no ports found under ports/");
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(text: &str) -> Result<Manifest> {
        Ok(toml::from_str(text)?)
    }

    #[test]
    fn a_minimal_manifest_parses() {
        let parsed = manifest(
            r#"
title = "Example"
description = "An example."
[render]
matrix = ["flavor"]
filename = "dist/acid-{flavor}.conf"
"#,
        )
        .unwrap();
        assert_eq!(parsed.title, "Example");
        assert!(parsed.preview.is_none());
        assert!(parsed.repo.is_none());
    }

    /// The schema is the point: a typo should be refused, not ignored.
    #[test]
    fn an_unknown_field_is_refused() {
        let parsed = manifest(
            r#"
title = "Example"
description = "An example."
nonsense = true
[render]
matrix = ["flavor"]
filename = "dist/x"
"#,
        );
        assert!(parsed.is_err());
    }

    #[test]
    fn a_missing_render_section_is_refused() {
        assert!(manifest("title = \"Example\"\ndescription = \"An example.\"\n").is_err());
    }

    #[test]
    fn a_value_can_be_shared_or_given_per_variant() {
        let shared = PerVariant::Same("one".to_owned());
        assert_eq!(shared.get("theme"), "one");
        assert_eq!(shared.get("anything"), "one");

        let parsed: Manifest = toml::from_str(
            r#"
title = "Example"
description = "An example."
[render]
matrix = ["flavor", { format = ["theme", "conf"] }]
filename = { theme = "a", conf = "b" }
"#,
        )
        .unwrap();
        assert_eq!(parsed.render.filename.get("theme"), "a");
        assert_eq!(parsed.render.filename.get("conf"), "b");
        // A variant the map does not mention has no filename at all.
        assert_eq!(parsed.render.filename.get("missing"), "");
    }

    #[test]
    fn the_repository_defaults_to_the_directory_name() {
        let config = Config {
            org: "acid-theme".to_owned(),
            hub: "acid-theme/acid".to_owned(),
            hub_ref: "main".to_owned(),
        };
        let mut port = Port {
            name: "rofi".to_owned(),
            dir: PathBuf::from("ports/rofi"),
            manifest: manifest(
                "title = \"rofi\"\ndescription = \"x\"\n[render]\nmatrix = [\"flavor\"]\nfilename = \"dist/x\"\n",
            )
            .unwrap(),
        };
        assert_eq!(port.slug(&config), "acid-theme/rofi");

        port.manifest.repo = Some("rofi-theme".to_owned());
        assert_eq!(port.slug(&config), "acid-theme/rofi-theme");
    }
}
