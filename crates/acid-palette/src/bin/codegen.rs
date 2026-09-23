//! Serialises the palette to `palette.json` at the repository root.
//!
//! The JSON is a committed build artefact: ports written in other languages and
//! the `acidify` renderer read it, so it must never be edited by hand. Run
//! `make palette` after touching `lib.rs`.

use std::{env, fs, path::PathBuf, process::ExitCode};

use acid_palette::{FLAVORS, json};

fn main() -> ExitCode {
    let mut contents = serde_json::to_string_pretty(&json::palette())
        .expect("the palette is plain data and always serialises");
    contents.push('\n');

    let destination = env::args().nth(1).map_or_else(
        || {
            // CARGO_MANIFEST_DIR is crates/acid-palette; the root is two up.
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("palette.json")
        },
        PathBuf::from,
    );

    if let Err(error) = fs::write(&destination, &contents) {
        eprintln!("codegen: cannot write {}: {error}", destination.display());
        return ExitCode::FAILURE;
    }

    println!(
        "codegen: wrote {} flavours to {}",
        FLAVORS.len(),
        destination.display()
    );
    ExitCode::SUCCESS
}
