// mako parses its config before it touches Wayland or DBus, so failing to start
// on either means the config was read successfully.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["mako"]) {
        return Ok(());
    }

    // Succeeds when mako gets past config parsing.
    let parses = |home: &str| {
        !exec("env", &["-u", "WAYLAND_DISPLAY", &format!("XDG_CONFIG_HOME={home}"),
                       "timeout", "5", "mako"])
            .contains("Failed to parse")
    };

    for flavour in h.flavours() {
        let theme = read(&h.dist(&format!("acid-{}.conf", flavour.identifier)));

        let root = std::env::temp_dir().join(flavour.identifier);
        std::fs::create_dir_all(root.join("mako"))?;
        std::fs::write(root.join("mako/config"), &theme)?;
        h.check(
            parses(&root.to_string_lossy()),
            format!("{}: mako parsed the theme", flavour.identifier),
        );

        // The theme is meant to be included, with the including file's own
        // globals after it. Criteria sections must not swallow them.
        let root = std::env::temp_dir().join(format!("{}-include", flavour.identifier));
        std::fs::create_dir_all(root.join("mako"))?;
        std::fs::write(root.join("mako/theme.conf"), &theme)?;
        std::fs::write(
            root.join("mako/config"),
            format!(
                "include={}/mako/theme.conf\nmax-visible=7\ndefault-timeout=10000\n",
                root.display()
            ),
        )?;
        h.check(
            parses(&root.to_string_lossy()),
            format!("{}: globals after the include are still globals", flavour.identifier),
        );
    }

    let root = std::env::temp_dir().join("mako-control");
    std::fs::create_dir_all(root.join("mako"))?;
    std::fs::write(root.join("mako/config"), "background-color=#000000\nnonsense-option=1\n")?;
    h.check(
        !parses(&root.to_string_lossy()),
        "control: mako rejects an unknown option",
    );

    Ok(())
}
