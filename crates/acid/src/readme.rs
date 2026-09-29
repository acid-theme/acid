//! The pieces a port's README needs, and the tables generated in the hub's.

use std::{fs, path::Path};

use anyhow::Result;

use crate::{
    manifest::{Config, Port},
    templates,
};

/// The files a port publishes, relative to its repository root.
pub fn published(port: &Port) -> Result<Vec<String>> {
    let dist = port.dist();
    let mut files = Vec::new();
    collect(&dist, &dist, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect(root: &Path, dir: &Path, into: &mut Vec<String>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, into)?;
        } else {
            into.push(path.strip_prefix(root)?.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

/// A row of preview images, one per flavour.
fn preview_table(port: &Port) -> String {
    if port.manifest.preview.is_none() {
        return String::new();
    }
    let names: Vec<&str> = acid_palette::FLAVORS.iter().map(|f| f.name).collect();
    let cells: Vec<String> = acid_palette::FLAVORS
        .iter()
        .map(|f| format!("![Acid {}](previews/{}.png)", f.name, f.identifier))
        .collect();
    format!(
        "| {} |\n| {} |\n| {} |",
        names.join(" | "),
        vec!["---"; names.len()].join(" | "),
        cells.join(" | ")
    )
}

pub fn render(config: &Config, port: &Port) -> Result<String> {
    Ok(templates::normalise(templates::readme(
        &port.name,
        &templates::Readme {
            hub: &config.hub,
            preview: &preview_table(port),
        },
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port(preview: bool) -> Port {
        let mut toml = r#"
            title = "Example"
            description = "Acid for Example."
            [render]
            matrix = ["flavor"]
            filename = "dist/acid-{flavor}.conf"
        "#
        .to_owned();
        if preview {
            toml.push_str(
                "
[preview]
packages = [\"example\"]
",
            );
        }
        Port {
            name: "example".to_owned(),
            dir: std::path::PathBuf::from("ports/example"),
            manifest: toml::from_str(&toml).expect("manifest parses"),
        }
    }

    /// The table has a column per flavour, and a control: a port with no
    /// preview gets nothing, so its README omits the section entirely.
    #[test]
    fn the_preview_table_covers_every_flavour() {
        let table = preview_table(&port(true));
        for flavour in &acid_palette::FLAVORS {
            assert!(
                table.contains(flavour.name),
                "{table} omits {}",
                flavour.name
            );
            assert!(table.contains(&format!("previews/{}.png", flavour.identifier)));
        }
        assert_eq!(preview_table(&port(false)), "");
    }
}
