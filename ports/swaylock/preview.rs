// The lock screen, with a few keystrokes sent so the ring shows its keypress
// segments rather than only the idle state.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    p.start_wayland("620x360", None)?;

    let theme = p.dist(&format!("acid-{}.conf", p.flavour.identifier));
    let theme = theme.to_string_lossy().into_owned();
    p.spawn_detached(
        "swaylock",
        &[
            "-C", &theme,
            "--indicator-idle-visible",
            "--indicator-radius", "70",
            "--indicator-thickness", "9",
            "--font", FONT,
        ],
        &[],
    )?;
    p.wait(5);
    let _ = p.call("wtype", &["acid"]);
    p.wait(1);
    p.capture_wayland()
}
