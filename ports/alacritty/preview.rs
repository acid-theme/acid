// The ANSI slots and a syntax-shaped sample, rendered by Alacritty itself.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    let ansi = p.root.join("previews/ansi.sh");
    let command = format!("bash {}; sleep 30", ansi.display());
    p.in_terminal("760x520", 84, 26, 11, &["bash", "-c", &command])?;
    p.capture_x11()
}
