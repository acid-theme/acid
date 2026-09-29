// Alacritty parses the theme itself. `migrate --dry-run` reports a parse error
// in its output but still exits 0, so the output is what gets checked.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["alacritty"]) {
        return Ok(());
    }

    let migrate = |path: &str| exec("alacritty", &["migrate", "--dry-run", "-c", path]);

    for flavour in h.flavours() {
        let theme = h.dist(&format!("acid-{}.toml", flavour.identifier));
        let output = migrate(&theme.to_string_lossy());

        h.check(
            !output.contains("migration failed"),
            format!("{}: alacritty parsed the theme", flavour.identifier),
        );
        for section in ["colors.primary", "colors.normal", "colors.bright", "colors.dim"] {
            h.check(
                output.contains(&format!("[{section}]")),
                format!("{}: [{section}] present", flavour.identifier),
            );
        }
    }

    let broken = scratch("broken.toml", "[colors.primary]\nbackground = \n")?;
    h.check(
        migrate(&broken.to_string_lossy()).contains("migration failed"),
        "control: alacritty rejects a broken theme",
    );

    Ok(())
}
