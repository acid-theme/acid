//! The generated documentation: the palette reference, and the tables in the
//! hub's README.

use std::{fs, path::Path};

use anyhow::{Context, Result};
use askama::Template;

use crate::{
    manifest::{Config, Port},
    templates,
};

const PORTS_START: &str = "<!-- ports:start -->";
const PORTS_END: &str = "<!-- ports:end -->";
const FLAVOURS_START: &str = "<!-- flavours:start -->";
const FLAVOURS_END: &str = "<!-- flavours:end -->";

pub fn palette_reference() -> Result<String> {
    let mut rendered = templates::Palette {
        version: acid_palette::VERSION,
        flavors: &acid_palette::FLAVORS,
    }
    .render()?;
    while rendered.ends_with('\n') {
        rendered.pop();
    }
    rendered.push('\n');
    Ok(rendered)
}

fn ports_table(config: &Config, ports: &[Port]) -> String {
    let rows: Vec<String> = ports
        .iter()
        .map(|port| {
            let slug = port.slug(config);
            format!(
                "| {} | [{slug}](https://github.com/{slug}) | [ports/{name}](ports/{name}) |",
                port.manifest.title,
                name = port.name
            )
        })
        .collect();
    format!(
        "{PORTS_START}\n\n| Port | Repository | Template |\n| --- | --- | --- |\n{}\n\n{PORTS_END}",
        rows.join("\n")
    )
}

fn flavours_table() -> String {
    let rows: Vec<String> = acid_palette::FLAVORS
        .iter()
        .map(|f| {
            format!(
                "| **{}** | `{}` | {} |",
                f.name, f.colors.base, f.description
            )
        })
        .collect();
    format!(
        "{FLAVOURS_START}\n\n| Flavour | Background | |\n| --- | --- | --- |\n{}\n\n{FLAVOURS_END}",
        rows.join("\n")
    )
}

fn replace_block(text: &str, start: &str, end: &str, block: &str) -> String {
    let (before, rest) = text.split_once(start).unwrap_or((text, ""));
    let (_, after) = rest.split_once(end).unwrap_or(("", ""));
    format!("{before}{block}{after}")
}

/// The hub README with its generated tables filled in.
pub fn hub_readme(root: &Path, config: &Config, ports: &[Port]) -> Result<String> {
    let path = root.join("README.md");
    let text =
        fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    for marker in [PORTS_START, PORTS_END, FLAVOURS_START, FLAVOURS_END] {
        if !text.contains(marker) {
            anyhow::bail!("README.md is missing the {marker} marker");
        }
    }
    let text = replace_block(&text, PORTS_START, PORTS_END, &ports_table(config, ports));
    Ok(replace_block(
        &text,
        FLAVOURS_START,
        FLAVOURS_END,
        &flavours_table(),
    ))
}

/// The generated documentation this repository keeps. A port's own README is
/// rendered when it is published, not stored here: it belongs to that port's
/// repository.
pub fn outputs(
    root: &Path,
    config: &Config,
    ports: &[Port],
) -> Result<Vec<(std::path::PathBuf, String)>> {
    Ok(vec![
        (root.join("docs/PALETTE.md"), palette_reference()?),
        (root.join("README.md"), hub_readme(root, config, ports)?),
    ])
}
