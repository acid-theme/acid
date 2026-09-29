//! Publishing each port to its own repository.
//!
//! A port's repository is a mirror: everything in it is assembled here and
//! pushed, except the previews, which that repository renders and commits
//! itself. Publishing is idempotent — a port whose files have not changed
//! produces no commit.

use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result, bail};

use crate::{
    manifest::{Config, Port},
    readme,
};

/// What a port's repository gets besides its themes, README and licence.
const WORKFLOW: &str = r#"name: Preview

# Generated. The steps live in the hub so every port shares them; this file only
# says when to run them.

on:
  push:
    paths:
%%WATCH%%
  workflow_dispatch:

jobs:
  preview:
    uses: %%HUB%%/.github/workflows/port-preview.yml@%%REF%%
    permissions:
      contents: write
"#;

fn run(args: &[&str], cwd: Option<&Path>) -> Result<String> {
    let mut command = Command::new(args[0]);
    command.args(&args[1..]);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    let output = command
        .output()
        .with_context(|| format!("cannot run {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "{} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn remote(config: &Config, port: &Port) -> String {
    let slug = port.slug(config);
    match std::env::var("ACID_PUBLISH_TOKEN") {
        Ok(token) if !token.is_empty() => {
            format!("https://x-access-token:{token}@github.com/{slug}.git")
        }
        _ => format!("https://github.com/{slug}.git"),
    }
}

/// Copy a directory's contents into another, creating it as needed.
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    for entry in fs::read_dir(from)? {
        let path = entry?.path();
        let target = to.join(path.file_name().context("a file with no name")?);
        if path.is_dir() {
            fs::create_dir_all(&target)?;
            copy_tree(&path, &target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

/// Empty the checkout, keeping .git and anything the port owns.
fn clear(repo: &Path, keep: &[&str]) -> Result<()> {
    for entry in fs::read_dir(repo)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name == ".git" || keep.contains(&name) {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

fn assemble(root: &Path, config: &Config, port: &Port, repo: &Path) -> Result<()> {
    copy_tree(&port.dist(), repo)?;
    fs::write(repo.join("README.md"), readme::render(config, port)?)?;
    fs::copy(root.join("LICENSE"), repo.join("LICENSE"))?;

    if port.manifest.preview.is_some() {
        let watch: String = readme::published(port)?
            .iter()
            .map(|f| format!("      - {f:?}"))
            .collect::<Vec<_>>()
            .join("\n");
        let workflow = WORKFLOW
            .replace("%%WATCH%%", &watch)
            .replace("%%HUB%%", &config.hub)
            .replace("%%REF%%", &config.hub_ref);
        let dir = repo.join(".github/workflows");
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("preview.yml"), workflow)?;
    }
    Ok(())
}

/// What happened to one port.
pub enum Outcome {
    Changed,
    Unchanged,
    NoRepository,
}

pub fn one(
    root: &Path,
    config: &Config,
    port: &Port,
    tag: Option<&str>,
    dry_run: bool,
) -> Result<Outcome> {
    let slug = port.slug(config);

    if dry_run {
        println!("  {slug}");
        for file in readme::published(port)? {
            println!("      {file}");
        }
        return Ok(Outcome::Changed);
    }

    let workdir = tempdir()?;
    let repo = workdir.join(&port.name);

    if let Err(error) = run(
        &[
            "git",
            "clone",
            "--quiet",
            &remote(config, port),
            &repo.to_string_lossy(),
        ],
        None,
    ) {
        let message = error.to_string().to_lowercase();
        if message.contains("not found") {
            println!("  {slug}: no repository yet, skipped");
            return Ok(Outcome::NoRepository);
        }
        return Err(error);
    }

    // A repository with no commits has an unborn HEAD named after whatever the
    // client defaults to, so the branch is chosen explicitly.
    let default = run(
        &[
            "git",
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
        Some(&repo),
    )
    .unwrap_or_default();
    let branch = default.strip_prefix("origin/").unwrap_or("main").to_owned();
    run(&["git", "checkout", "--quiet", "-B", &branch], Some(&repo))?;

    // The port renders its own previews; this only ever carries them over.
    clear(&repo, &["previews"])?;
    assemble(root, config, port, &repo)?;

    run(&["git", "add", "--all"], Some(&repo))?;
    if run(&["git", "status", "--porcelain"], Some(&repo))?.is_empty() {
        if let Some(tag) = tag {
            apply_tag(&repo, tag)?;
            println!("  {slug}: already up to date, tagged {tag}");
        } else {
            println!("  {slug}: already up to date");
        }
        return Ok(Outcome::Unchanged);
    }

    let hub = run(&["git", "rev-parse", "--short", "HEAD"], Some(root)).unwrap_or_default();
    let subject = match tag {
        Some(tag) => format!("Sync {tag} from {}@{hub}", config.hub),
        None => format!("Sync from {}@{hub}", config.hub),
    };
    run(
        &[
            "git",
            "-c",
            "user.name=acid-theme bot",
            "-c",
            "user.email=noreply@github.com",
            "commit",
            "--quiet",
            "--message",
            &subject,
        ],
        Some(&repo),
    )?;
    run(
        &[
            "git",
            "push",
            "--quiet",
            "origin",
            &format!("HEAD:{branch}"),
        ],
        Some(&repo),
    )?;
    if let Some(tag) = tag {
        apply_tag(&repo, tag)?;
    }
    println!("  {slug}: pushed {subject}");
    Ok(Outcome::Changed)
}

fn apply_tag(repo: &Path, tag: &str) -> Result<()> {
    run(&["git", "tag", "--force", tag], Some(repo))?;
    run(
        &["git", "push", "--quiet", "--force", "origin", tag],
        Some(repo),
    )?;
    Ok(())
}

fn tempdir() -> Result<std::path::PathBuf> {
    let base = std::env::temp_dir().join(format!("acid-publish-{}", std::process::id()));
    fs::create_dir_all(&base)?;
    Ok(base)
}
