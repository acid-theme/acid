//! Lists the ports for the template macro.
//!
//! askama compiles templates into the binary, so every one has to be named at
//! build time. Scanning `ports/` here keeps that list out of the source: adding
//! a port stays a matter of adding its folder.

use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ports = root.join("ports");
    println!("cargo::rerun-if-changed={}", ports.display());

    let mut entries: Vec<_> = fs::read_dir(&ports)
        .expect("ports/ should exist")
        .filter_map(Result::ok)
        .filter(|e| e.path().join("port.toml").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();

    let mut listed = String::from("acid_templates! {\n");
    for name in &entries {
        println!("cargo::rerun-if-changed={}", ports.join(name).display());
        let ident: String = name
            .split(['-', '_'])
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect();
        // askama needs a literal path in the attribute, so both are emitted.
        listed.push_str(&format!(
            "    {ident} => {name:?}, {:?}, {:?};\n",
            format!("{name}/theme.tera"),
            format!("{name}/README.tera"),
        ));
    }
    listed.push_str("}\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set"));
    fs::write(out_dir.join("ports.rs"), listed).expect("the port list should be writable");

    // Each port's test and preview are Rust, included from the port's folder so
    // everything about a port still lives in one place.
    for (kind, file, context) in [
        ("tests", "test.rs", "Harness"),
        ("previews", "preview.rs", "Preview"),
    ] {
        let mut module = String::new();
        let mut dispatch = String::new();
        for name in &entries {
            let path = ports.join(name).join(file);
            if !path.is_file() {
                continue;
            }
            let ident = name.replace('-', "_");
            module.push_str(&format!(
                "pub mod {ident} {{\n    use super::*;\n    include!({:?});\n}}\n",
                path.canonicalize().unwrap_or(path.clone())
            ));
            dispatch.push_str(&format!("        {name:?} => {ident}::run(h),\n"));
        }
        let body = format!(
            "{module}\npub fn run(h: &mut {context}) -> anyhow::Result<()> {{\n                 match h.port.as_str() {{\n{dispatch}        other =>              anyhow::bail!(\"no {kind} for port {{other}}\"),\n    }}\n}}\n"
        );
        fs::write(out_dir.join(format!("{kind}.rs")), body)
            .expect("the generated module should be writable");
    }
}
