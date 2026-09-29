// niri validates its own config. An included `layout` block merges with the
// config's, which a second one in the same file cannot — the port depends on
// that, so it is asserted here.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["niri"]) {
        return Ok(());
    }

    let validate = |path: &str| exec("niri", &["validate", "-c", path]);

    for flavour in h.flavours() {
        let theme = h.dist(&format!("acid-{}.kdl", flavour.identifier));
        h.check(
            validate(&theme.to_string_lossy()).contains("config is valid"),
            format!("{}: niri validated the theme", flavour.identifier),
        );
    }

    // The port's premise: an include and a local layout block coexist.
    let theme = h.dist("acid-acetic.kdl");
    let config = scratch(
        "config.kdl",
        &format!(
            "include {:?}\nlayout {{\n    gaps 4\n    border {{\n        width 0.5\n    }}\n}}\n",
            theme.to_string_lossy()
        ),
    )?;
    h.check(
        validate(&config.to_string_lossy()).contains("config is valid"),
        "an included layout block merges with a local one",
    );

    // Two in one file must still be an error, or the above proves nothing.
    let duplicate = scratch("dup.kdl", "layout {\n    gaps 4\n}\nlayout {\n    gaps 8\n}\n")?;
    h.check(
        validate(&duplicate.to_string_lossy()).contains("duplicate node"),
        "control: two layout blocks in one file are rejected",
    );

    Ok(())
}
