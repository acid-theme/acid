//! The pieces a port's README needs, and the tables generated in the hub's.

use std::{fs, path::Path};

use anyhow::Result;

use crate::{
    manifest::{Config, Port},
    templates,
};

/// One line naming every flavour, so no prose has to count them.
pub fn flavour_summary() -> String {
    let parts: Vec<String> = acid_palette::FLAVORS
        .iter()
        .map(|f| format!("**{}** (`{}`), {}", f.name, f.colors.base, f.description))
        .collect();

    let lead = match parts.len() {
        1 => "One flavour".to_owned(),
        2 => "Two flavours".to_owned(),
        3 => "Three flavours".to_owned(),
        n => format!("{n} flavours"),
    };
    let joined = match parts.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{}; and {last}", rest.join("; ")),
        None => String::new(),
    };
    format!("{lead}: {joined}.")
}

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
    let files = published(port)?;
    let listing = files
        .iter()
        .map(|f| format!("- `{f}`"))
        .collect::<Vec<_>>()
        .join("\n");

    templates::readme(
        &port.name,
        &templates::Readme {
            hub: &config.hub,
            version: acid_palette::VERSION,
            flavours: &flavour_summary(),
            files: &listing,
            template: &format!("ports/{}/theme.tera", port.name),
            preview: &preview_table(port),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The summary counts the flavours so no prose has to.
    #[test]
    fn the_summary_names_every_flavour() {
        let summary = flavour_summary();
        for flavour in &acid_palette::FLAVORS {
            assert!(
                summary.contains(flavour.name),
                "{summary} omits {}",
                flavour.name
            );
            assert!(summary.contains(flavour.description));
        }
        assert!(summary.starts_with("Three flavours:"), "{summary}");
        // The last is joined with "and", not another semicolon.
        assert!(summary.contains("; and "), "{summary}");
    }
}
