// Two notifications at different urgencies, so the accent escalation shows.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    p.start_wayland("460x180", Some(&p.flavour.colors.base.to_hex_string()))?;

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_owned());
    let config = std::path::Path::new(&home).join(".config/mako");
    std::fs::create_dir_all(&config)?;
    let mut theme = std::fs::read_to_string(p.dist(&format!(
        "acid-{}.conf",
        p.flavour.identifier
    )))?;
    theme.push_str(&format!(
        "font={FONT} 11\nborder-size=2\nborder-radius=4\npadding=12\n\
         margin=12\nwidth=420\ndefault-timeout=0\n"
    ));
    std::fs::write(config.join("config"), theme)?;

    // mako needs a session bus, and the notifications need mako running.
    let script = format!(
        "mako & sleep 2; \
         notify-send -u normal 'Build finished' 'cargo test: 54 passed, 0 failed'; \
         notify-send -u critical 'Disk almost full' '3% remaining on /home'; \
         sleep 3; grim {}",
        p.out.display()
    );
    p.call("dbus-run-session", &["--", "bash", "-c", &script])
}
