// The bar as Waybar draws it, on an output exactly its own height.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    p.start_wayland("900x34", None)?;

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_owned());
    let config = std::path::Path::new(&home).join(".config/waybar");
    std::fs::create_dir_all(&config)?;
    std::fs::copy(
        p.dist(&format!("acid-{}.css", p.flavour.identifier)),
        config.join("theme.css"),
    )?;
    std::fs::write(config.join("config"), include_str!("preview.json"))?;
    std::fs::write(
        config.join("style.css"),
        format!(include_str!("preview.css"), font = FONT),
    )?;

    let script = format!("waybar & sleep 5; grim {}", p.out.display());
    p.call("dbus-run-session", &["--", "bash", "-c", &script])
}
