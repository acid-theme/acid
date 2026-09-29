// btop running for real, so the meters, gradients and box outlines are its own
// rendering rather than a mock-up.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_owned());
    let config = std::path::Path::new(&home).join(".config/btop");
    std::fs::create_dir_all(config.join("themes"))?;
    std::fs::copy(
        p.dist(&format!("acid-{}.theme", p.flavour.identifier)),
        config.join(format!("themes/acid-{}.theme", p.flavour.identifier)),
    )?;
    std::fs::write(
        config.join("btop.conf"),
        format!(
            "color_theme = \"acid-{}\"\ntheme_background = True\ntruecolor = True\n\
             vim_keys = False\nrounded_corners = True\ngraph_symbol = \"braille\"\n\
             shown_boxes = \"cpu mem net proc\"\nupdate_ms = 500\nproc_sorting = \"cpu lazy\"\n",
            p.flavour.identifier
        ),
    )?;

    p.in_terminal("1100x700", 130, 40, 9, &["btop"])?;
    // btop needs a moment to fill its graphs with something other than zeroes.
    p.wait(6);
    p.capture_x11()
}
