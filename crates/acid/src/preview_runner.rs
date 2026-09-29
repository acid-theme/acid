//! Rendering a port's preview, one image per flavour.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::{
    image,
    manifest::{Config, Port},
};

/// Render every flavour of one port. Returns the images written.
pub fn one(root: &Path, config: &Config, port: &Port) -> Result<Vec<PathBuf>> {
    let built = image::prepare(config, port)?;
    image::publish(&built)?;

    let out = image::preview_dir(root, port);
    std::fs::create_dir_all(&out)?;

    let mut written = Vec::new();
    for flavour in &acid_palette::FLAVORS {
        let path = out.join(format!("{}.png", flavour.identifier));
        print!("  {:<16} {} ... ", port.name, flavour.identifier);
        use std::io::Write;
        std::io::stdout().flush().ok();

        let ok = image::run(
            &built,
            root,
            &["preview-inside", &port.name],
            &[
                ("FLAVOUR", flavour.identifier),
                ("OUT", &format!("/out/{}.png", flavour.identifier)),
            ],
            Some(&out),
        )?;
        if ok && path.is_file() {
            println!("{}", describe(&path));
            written.push(path);
        } else {
            println!("FAILED");
        }
    }
    Ok(written)
}

fn describe(path: &Path) -> String {
    std::process::Command::new("identify")
        .args(["-format", "%wx%h"])
        .arg(path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "written".to_owned())
}

/// The flavour named by the environment, for the inside half.
pub fn flavour_from_env() -> Result<&'static acid_palette::Flavor> {
    let name = std::env::var("FLAVOUR").context("FLAVOUR is not set")?;
    acid_palette::FLAVORS
        .iter()
        .find(|f| f.identifier == name)
        .with_context(|| format!("no such flavour: {name}"))
}
