// rofi parses the theme itself. `-dump-theme` reports a parse failure on
// stderr but still exits 0, so the output is what gets checked.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["rofi"]) {
        return Ok(());
    }

    let roles: Vec<&str> = acid_palette::ACETIC
        .colors
        .iter()
        .map(|c| c.identifier)
        .collect();

    let parse = |path: &str| exec("rofi", &["-no-config", "-theme", path, "-dump-theme"]);

    for flavour in h.flavours() {
        let theme = h.dist(&format!("acid-{}.rasi", flavour.identifier));
        let path = theme.to_string_lossy().into_owned();

        let rejected = parse(&path).contains("Failed to parse");
        h.check(!rejected, format!("{}: rofi parsed the theme", flavour.identifier));

        // rofi accepts an undefined name silently, so it cannot report a
        // missing role. The file is read instead.
        let contents = read(&theme);
        let missing: Vec<&str> = roles
            .iter()
            .copied()
            .filter(|role| !contents.contains(&format!("    {role}: ")))
            .collect();
        h.check(
            missing.is_empty(),
            format!(
                "{}: all {} roles defined{}",
                flavour.identifier,
                roles.len(),
                if missing.is_empty() { String::new() } else { format!("; missing {missing:?}") }
            ),
        );

        // A theme importing this one and using every role must still parse.
        let mut body = format!("@import {:?}\n", path.trim_end_matches(".rasi"));
        for (index, role) in roles.iter().enumerate() {
            body.push_str(&format!("element-{index} {{ background-color: @{role}; }}\n"));
        }
        let probe = scratch(&format!("probe-{}.rasi", flavour.identifier), &body)?;
        let rejected = parse(&probe.to_string_lossy()).contains("Failed to parse");
        h.check(
            !rejected,
            format!("{}: imports and resolves in another theme", flavour.identifier),
        );
    }

    // Documented behaviour: an unknown name is not an error, which is why the
    // role check reads the file. The name avoids an underscore, which is a
    // syntax error in rasi and would test the wrong thing.
    let undefined = scratch("undefined.rasi", "window { background-color: @nosuchrole; }\n")?;
    let rejected = parse(&undefined.to_string_lossy()).contains("Failed to parse");
    h.check(!rejected, "an undefined variable is accepted silently, as documented");

    // The control proves the parse check can fail.
    let broken = scratch("broken.rasi", "* {\n  base: #00zz00;\n")?;
    let rejected = parse(&broken.to_string_lossy()).contains("Failed to parse");
    h.check(rejected, "control: rofi rejects a broken theme");

    Ok(())
}
