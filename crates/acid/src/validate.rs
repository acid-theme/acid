//! What `acid check` enforces beyond the generated files being current.
//!
//! Each of these was added because something slipped past: a theme published
//! without a version stamp, and a port that named preview packages but had no
//! preview to run.

use anyhow::Result;

use crate::{manifest::Port, readme, render};

/// Returns the problems found, empty when there are none.
pub fn ports(ports: &[Port]) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    let version = acid_palette::VERSION;

    for port in ports {
        // A preview needs both a script to run and the packages to run it with.
        let script = port.preview_script().is_file();
        let packages = port.manifest.preview.is_some();
        if script && !packages {
            problems.push(format!(
                "{}: has preview.rs but no [preview] packages",
                port.name
            ));
        }
        if packages && !script {
            problems.push(format!(
                "{}: has [preview] packages but no preview.rs",
                port.name
            ));
        }

        if !port.test().is_file() {
            problems.push(format!("{}: has no test.rs", port.name));
        }

        // Every published file names the version that produced it, so a theme
        // installed by hand can be identified.
        for file in readme::published(port)? {
            let path = port.dist().join(&file);
            let contents = std::fs::read(&path).unwrap_or_default();
            let text = String::from_utf8_lossy(&contents);
            if !text.contains(version) {
                problems.push(format!(
                    "{}: {file} is not stamped with version {version}",
                    port.name
                ));
            }
        }

        // A filename given per variant must cover every value of the axis.
        for variant in render::variants(port)? {
            if render::path(port, &variant)
                .file_name()
                .is_none_or(|name| name.is_empty())
            {
                problems.push(format!(
                    "{}: no filename for variant {:?}",
                    port.name, variant.format
                ));
            }
        }
    }

    Ok(problems)
}
