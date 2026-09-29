mod docs;
mod filters;
mod harness;
mod image;
mod manifest;
mod preview;
mod preview_runner;
mod previews;
mod publish;
mod readme;
mod render;
mod scaffold;
mod templates;
mod tests;
mod validate;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Build, check and publish Acid and its ports.")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start a new port: its folder, manifest, template, README and test.
    New {
        /// The port's name, which is also its directory and repository.
        name: String,
        /// The display name, e.g. "Telegram Desktop". Defaults to the name.
        #[arg(long)]
        title: Option<String>,
    },
    /// Render every port's theme into its dist directory.
    Render,
    /// Render the palette reference, the hub's tables and each port's README.
    Docs,
    /// Fail if anything generated is out of date.
    Check,
    /// Check each port with its own program, in a container. Needs podman.
    Test {
        /// Limit to these ports.
        ports: Vec<String>,
    },
    /// Run one port's checks. Used inside the container.
    #[command(hide = true)]
    TestInside { port: String },
    /// Render a preview of each port by running the real program. Needs podman.
    Preview {
        /// Limit to these ports.
        ports: Vec<String>,
    },
    /// Render one preview. Used inside the container.
    #[command(hide = true)]
    PreviewInside { port: String },
    /// Push each port to its own repository.
    Publish {
        /// Limit to these ports.
        #[arg(long)]
        only: Vec<String>,
        /// Also move this tag in each mirror. Must match the palette's version.
        #[arg(long)]
        tag: Option<String>,
        /// Report what would be published without pushing.
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Command::TestInside { port } => {
            let mut harness = harness::Harness::new(PathBuf::from("/acid"), port.clone());
            tests::run(&mut harness)?;
            harness.summary();
            if harness.failed() {
                std::process::exit(1);
            }
            return Ok(());
        }
        Command::PreviewInside { port } => {
            let flavour = preview_runner::flavour_from_env()?;
            let out = PathBuf::from(std::env::var("OUT").unwrap_or_else(|_| "/out.png".into()));
            let mut context =
                preview::Preview::new(PathBuf::from("/acid"), port.clone(), flavour, out);
            previews::run(&mut context)?;
            return Ok(());
        }
        _ => {}
    }

    let root = manifest::root();
    let config = manifest::config(&root)?;
    let ports = manifest::ports(&root)?;

    match args.command {
        Command::TestInside { .. } | Command::PreviewInside { .. } => unreachable!(),
        Command::New { name, title } => {
            let title = title.unwrap_or_else(|| name.clone());
            scaffold::port(&root, &name, &title)?;
        }
        Command::Render => {
            let written = render::all(&ports, true)?;
            println!(
                "render: {} file(s) written, {} ports",
                written.len(),
                ports.len()
            );
        }
        Command::Docs => {
            let mut written = 0;
            for (path, contents) in docs::outputs(&root, &config, &ports)? {
                if std::fs::read_to_string(&path).ok().as_deref() != Some(contents.as_str()) {
                    std::fs::write(&path, contents)?;
                    written += 1;
                }
            }
            println!("docs: {written} file(s) written");
        }
        Command::Check => {
            let mut stale = render::all(&ports, false)?;
            for (path, contents) in docs::outputs(&root, &config, &ports)? {
                if std::fs::read_to_string(&path).ok().as_deref() != Some(contents.as_str()) {
                    stale.push(path);
                }
            }
            let problems = validate::ports(&ports)?;
            if !problems.is_empty() {
                for problem in &problems {
                    eprintln!("  {problem}");
                }
                anyhow::bail!("{} problem(s) found", problems.len());
            }
            if !stale.is_empty() {
                for path in &stale {
                    eprintln!("  stale: {}", path.display());
                }
                anyhow::bail!(
                    "{} generated file(s) are out of date; run `make`",
                    stale.len()
                );
            }
            println!("check: {} ports, everything current", ports.len());
        }
        Command::Test { ports: only } => {
            let selected: Vec<_> = ports
                .iter()
                .filter(|p| only.is_empty() || only.contains(&p.name))
                .filter(|p| p.test().is_file())
                .collect();

            let mut failed = false;
            for port in selected {
                let built = image::prepare(&config, port)?;
                image::publish(&built)?;
                println!("== {}", port.name);
                if !image::run(&built, &root, &["test-inside", &port.name], &[], None)? {
                    failed = true;
                }
            }
            if failed {
                anyhow::bail!("some ports failed");
            }
            println!("tests: all ports passed");
        }
        Command::Preview { ports: only } => {
            let selected: Vec<_> = ports
                .iter()
                .filter(|p| only.is_empty() || only.contains(&p.name))
                .filter(|p| p.preview_script().is_file())
                .collect();

            for port in selected {
                preview_runner::one(&root, &config, port)?;
            }
        }
        Command::Publish { only, tag, dry_run } => {
            // The version in every generated file comes from the palette, so a
            // tag that disagrees with it would label the release wrongly.
            if let Some(tag) = &tag {
                let expected = format!("v{}", acid_palette::VERSION);
                if tag != &expected {
                    anyhow::bail!(
                        "tag {tag} does not match the palette's version, which is {expected}"
                    );
                }
            }

            let selected: Vec<_> = ports
                .iter()
                .filter(|p| only.is_empty() || only.contains(&p.name))
                .collect();
            if selected.is_empty() {
                anyhow::bail!("no such port: {}", only.join(", "));
            }

            println!(
                "{} {} port(s) to {}:",
                if dry_run {
                    "would publish"
                } else {
                    "publishing"
                },
                selected.len(),
                config.org
            );

            let (mut changed, mut missing) = (0, 0);
            for port in selected {
                match publish::one(&root, &config, port, tag.as_deref(), dry_run)? {
                    publish::Outcome::Changed => changed += 1,
                    publish::Outcome::NoRepository => missing += 1,
                    publish::Outcome::Unchanged => {}
                }
            }
            if !dry_run {
                print!("{changed} port(s) changed");
                if missing > 0 {
                    print!(", {missing} without a repository yet");
                }
                println!();
            }
        }
    }

    Ok(())
}
