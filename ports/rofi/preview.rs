// rofi drawing its own menu. The layout follows a conventional rofi theme —
// prompt, filter counters, dashed separator, alternating rows — so the preview
// shows the roles doing real work rather than a bare list.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    p.start_x11("620x420")?;

    let theme = p.dist(&format!("acid-{}.rasi", p.flavour.identifier));
    let import = theme.to_string_lossy().trim_end_matches(".rasi").to_owned();
    let config = std::env::temp_dir().join("preview.rasi");
    std::fs::write(
        &config,
        format!(
            include_str!("preview.rasi"),
            import = import,
            font = FONT
        ),
    )?;

    let entries = "alacritty\nnvim ~/sources/acid\nfirefox\nwaybar\n\
                   systemctl --user restart mako\nswaylock -f\n";
    let list = std::env::temp_dir().join("entries");
    std::fs::write(&list, entries)?;

    // -a marks a row active and -u marks one urgent, so both states show.
    let command = format!(
        "rofi -dmenu -a 2 -u 5 -theme {} -p run < {} > /dev/null 2>&1",
        config.display(),
        list.display()
    );
    p.spawn_detached("bash", &["-c", &command], &[])?;
    p.wait(4);

    let found = std::process::Command::new("xdotool")
        .args(["search", "--class", "rofi"])
        .output()?;
    p.window = String::from_utf8_lossy(&found.stdout)
        .lines()
        .next_back()
        .map(str::to_owned);
    p.capture_x11()
}
