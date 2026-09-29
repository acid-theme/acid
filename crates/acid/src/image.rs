//! The container images the checks and previews run in.
//!
//! One image per port, built from the packages that port's manifest names, so a
//! run installs that port's tool rather than every port's. The image is named
//! after a hash of its own definition, and pulled from the registry when one has
//! been published, so an unchanged definition is never rebuilt.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::manifest::{Config, Port};

/// What every image needs regardless of the port.
const BASE: &[&str] = &["python", "imagemagick", "ttf-jetbrains-mono-nerd"];

/// Whether the image had to be built, which is what CI publishes.
#[derive(PartialEq, Eq)]
pub enum Source {
    Cached,
    Pulled,
    Built,
}

pub struct Image {
    pub reference: String,
    pub remote: String,
    pub source: Source,
}

fn definition(port: &Port, packages: &[String]) -> String {
    let listed: Vec<&str> = packages
        .iter()
        .map(String::as_str)
        .chain(BASE.iter().copied())
        .collect();
    format!(
        "# Generated. Everything the {name} checks and preview need, and nothing\n\
         # another port needs.\n\
         FROM docker.io/library/archlinux:latest\n\n\
         # The Arch image drops man pages to save space, but a port may be checked\n\
         # against documentation its own program ships.\n\
         RUN sed -i '/^NoExtract.*usr\\/share\\/man/d' /etc/pacman.conf\n\n\
         RUN pacman -Syu --noconfirm --needed --disable-download-timeout \\\n        {} \\\n    \
         && pacman -Scc --noconfirm\n\n\
         # Rootless podman refuses to exec a binary carrying an effective capability\n\
         # set, so they are stripped. sway and btop both ship with one.\n\
         RUN getcap -r /usr/bin /usr/sbin 2>/dev/null | cut -d' ' -f1 | xargs -r -n1 setcap -r || true\n\n\
         # Waybar will not start without a machine id. A no-op when dbus is absent.\n\
         RUN command -v dbus-uuidgen >/dev/null && dbus-uuidgen --ensure=/etc/machine-id || true\n\n\
         WORKDIR /acid\n\
         ENV XDG_RUNTIME_DIR=/tmp/xdg\n",
        listed.join(" \\\n        "),
        name = port.name
    )
}

/// A hash of the definition, so a changed one means a different image.
fn tag(definition: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in definition.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Every package the port needs for both its checks and its preview, so one
/// image serves both.
fn packages(port: &Port) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    for section in [port.manifest.test.as_ref(), port.manifest.preview.as_ref()] {
        for package in section.map(|s| s.packages.as_slice()).unwrap_or_default() {
            if !all.contains(package) {
                all.push(package.clone());
            }
        }
    }
    all.sort();
    all
}

pub fn prepare(config: &Config, port: &Port) -> Result<Image> {
    let definition = definition(port, &packages(port));
    let tag = tag(&definition);
    let reference = format!("acid-{}:{tag}", port.name);
    let remote = format!("ghcr.io/{}/acid-{}:{tag}", config.org, port.name);

    if podman(&["image", "exists", &reference])? {
        return Ok(Image {
            reference,
            remote,
            source: Source::Cached,
        });
    }
    if podman(&["pull", "--quiet", &remote])? {
        podman(&["tag", &remote, &reference])?;
        return Ok(Image {
            reference,
            remote,
            source: Source::Pulled,
        });
    }

    let dir = std::env::temp_dir().join(format!("acid-image-{}", port.name));
    std::fs::create_dir_all(&dir)?;
    let file = dir.join("Containerfile");
    std::fs::write(&file, &definition)?;

    println!("building {reference}");
    let built = std::process::Command::new("podman")
        .args(["build", "-t", &reference, "-t", &remote, "-f"])
        .arg(&file)
        .arg(&dir)
        .status()
        .context("podman is not installed")?;
    if !built.success() {
        bail!("could not build {reference}");
    }
    Ok(Image {
        reference,
        remote,
        source: Source::Built,
    })
}

/// Push an image the registry does not have yet.
pub fn publish(image: &Image) -> Result<()> {
    if image.source != Source::Built {
        return Ok(());
    }
    if !podman(&["push", &image.remote])? {
        // A push failing is not a failing check; the next run rebuilds.
        eprintln!("could not push {}", image.remote);
    }
    Ok(())
}

fn podman(args: &[&str]) -> Result<bool> {
    Ok(std::process::Command::new("podman")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("podman is not installed")?
        .success())
}

/// Run this binary inside the image.
pub fn run(
    image: &Image,
    root: &Path,
    args: &[&str],
    env: &[(&str, &str)],
    out: Option<&Path>,
) -> Result<bool> {
    // Copied before mounting: the running binary is replaced by a rebuild, and
    // a long run would otherwise fail part way through with the mount pointing
    // at a deleted file.
    let binary = staged_binary()?;
    let mut command = std::process::Command::new("podman");
    command
        .args(["run", "--rm"])
        .arg("-v")
        .arg(format!("{}:/acid:ro,Z", root.display()))
        .arg("-v")
        .arg(format!("{}:/acid-bin:ro,Z", binary.display()));
    if let Some(out) = out {
        command.arg("-v").arg(format!("{}:/out:Z", out.display()));
    }
    for (key, value) in env {
        command.arg("-e").arg(format!("{key}={value}"));
    }
    command.arg(&image.reference).arg("/acid-bin").args(args);
    Ok(command.status()?.success())
}

/// A copy of this binary in a stable place, made once per run.
fn staged_binary() -> Result<PathBuf> {
    let source = std::env::current_exe().context("cannot find the running binary")?;
    let staged = std::env::temp_dir().join(format!("acid-bin-{}", std::process::id()));
    if !staged.exists() {
        std::fs::copy(&source, &staged)?;
    }
    Ok(staged)
}

/// Where a port's rendered previews are written.
pub fn preview_dir(root: &Path, port: &Port) -> PathBuf {
    root.join("target/previews").join(&port.name)
}
