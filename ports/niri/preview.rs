// niri nested in a headless compositor, with two windows so the active and
// inactive border colours are both visible. The config is the documented one:
// the theme included, the geometry local.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    p.start_wayland("900x420", None)?;

    let alacritty = std::env::temp_dir().join("alacritty.toml");
    std::fs::write(
        &alacritty,
        format!(
            "general.import = [{:?}]\n[font]\nsize = 8\n[font.normal]\nfamily = {FONT:?}\n",
            p.dist_of("alacritty", &format!("acid-{}.toml", p.flavour.identifier))
                .to_string_lossy()
        ),
    )?;

    let config = std::env::temp_dir().join("niri.kdl");
    std::fs::write(
        &config,
        format!(
            "include {theme:?}\n\n\
             layout {{\n    gaps 10\n    border {{\n        width 3\n    }}\n}}\n\n\
             prefer-no-csd\n\n\
             spawn-at-startup \"alacritty\" \"--config-file\" {term:?} \"-e\" \"nvim\" \
             \"-c\" \"set number cursorline\" \"-c\" \"lua pcall(vim.treesitter.start)\" {sample:?}\n\
             spawn-at-startup \"alacritty\" \"--config-file\" {term:?} \"-e\" \"bash\" \"-c\" {ansi:?}\n\n\
             hotkey-overlay {{\n    skip-at-startup\n}}\n",
            theme = p.dist(&format!("acid-{}.kdl", p.flavour.identifier)).to_string_lossy(),
            term = alacritty.to_string_lossy(),
            sample = p.root.join("previews/sample.lua").to_string_lossy(),
            ansi = format!("bash {}; sleep 60", p.root.join("previews/ansi.sh").display()),
        ),
    )?;

    p.spawn_detached("niri", &["-c", &config.to_string_lossy()], &[])?;
    p.wait(8);
    p.capture_wayland()
}
