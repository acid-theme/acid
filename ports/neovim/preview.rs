// A source file open in Neovim, with the line number column and cursor line
// visible so the fill and overlay roles show. Lua, because Neovim bundles that
// parser — the port is mostly treesitter groups, so the preview must use them.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    let rtp = format!("set rtp^={}", p.dist("").display());
    let scheme = format!("colorscheme acid-{}", p.flavour.identifier);
    let sample = p.root.join("previews/sample.lua");
    let sample = sample.to_string_lossy().into_owned();

    p.in_terminal(
        "760x520",
        84,
        26,
        11,
        &[
            "nvim",
            "--cmd", &rtp,
            "-c", &scheme,
            "-c", "set number cursorline signcolumn=yes laststatus=2",
            "-c", "lua pcall(vim.treesitter.start)",
            "-c", "normal 11G",
            &sample,
        ],
    )?;
    p.capture_x11()
}
