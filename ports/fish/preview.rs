// A command line typed into fish, so the colours are the ones fish applies as
// text is entered rather than a printed imitation.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    let rc = std::env::temp_dir().join("fishrc.fish");
    std::fs::write(
        &rc,
        format!(
            "source {}\nset -g fish_greeting \"\"\n\
             function fish_prompt\n    set_color $fish_color_cwd\n    \
             echo -n \"~/sources/acid\"\n    set_color normal\n    echo -n \" > \"\nend\n",
            p.dist(&format!("conf.d/acid-{}.fish", p.flavour.identifier)).display()
        ),
    )?;

    let source = format!("source {}", rc.display());
    p.in_terminal("820x400", 84, 6, 11, &["fish", "-C", &source])?;

    // There is no window manager under Xvfb, so focus has to be set first.
    if let Some(window) = p.window.clone() {
        p.call("xdotool", &["windowfocus", "--sync", &window])?;
    }
    p.call("xdotool", &["type", "--delay", "40", "echo \"acid\" | string upper"])?;
    p.call("xdotool", &["key", "Return"])?;
    p.wait(1);
    // Left uncommitted: this is the line whose highlighting is on display.
    p.call("xdotool", &["type", "--delay", "40",
                        "grep -rn --color=auto \"fish_color\" conf.d/ > /tmp/hits.txt"])?;
    p.wait(1);
    p.capture_x11()
}
