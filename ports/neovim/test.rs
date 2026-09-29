// Neovim loads the colourscheme and reports what actually resolved.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["nvim"]) {
        return Ok(());
    }

    let rtp = h.root.join("ports/neovim/dist");
    let rtp = format!("set rtp^={}", rtp.display());

    // Run a Lua snippet against a loaded flavour and return its output.
    let probe = |flavour: &str, lua: &str| -> String {
        exec(
            "nvim",
            &[
                "--headless", "--clean", "-u", "NONE",
                "--cmd", &rtp,
                "-c", &format!("colorscheme acid-{flavour}"),
                "-c", &format!("lua {lua}"),
                "-c", "qa!",
            ],
        )
    };

    let captures: Vec<String> = read(std::path::Path::new(
        "/usr/share/nvim/runtime/doc/treesitter.txt",
    ))
    .lines()
    .filter(|line| line.starts_with('@'))
    .filter_map(|line| line.split_whitespace().next())
    .filter(|name| *name != "@none")
    .map(str::to_owned)
    .collect::<std::collections::BTreeSet<_>>()
    .into_iter()
    .collect();

    if captures.len() < 50 {
        h.fail(format!("could not read Neovim's capture list ({})", captures.len()));
        return Ok(());
    }
    h.note(format!("Neovim documents {} captures", captures.len()));

    let groups = captures.join(" ");
    let plugins = "GitSignsAdd MiniDiffSignAdd MiniFilesNormal MiniPickNormal \
        MiniPickMatchRanges MiniStatuslineModeNormal MiniHipatternsTodo \
        MiniIndentscopeSymbol MiniSurround MiniIconsAzure BlinkCmpMenu \
        BlinkCmpLabelMatch BlinkCmpKindFunction CmpItemAbbrMatch CmpItemKindFunction \
        TelescopeNormal TelescopeSelection TelescopeMatching IblIndent IblScope \
        CopilotSuggestion ObsidianTag";

    for flavour in h.flavours() {
        let id = flavour.identifier;

        let output = probe(id, "print('NAME ' .. tostring(vim.g.colors_name))");
        h.check(
            output.contains(&format!("NAME acid-{id}")),
            format!("{id}: colourscheme loaded and named itself"),
        );

        let output = probe(
            id,
            "local n = 0 \
             for _, hl in pairs(vim.api.nvim_get_hl(0, {})) do \
               for _, a in ipairs({'fg', 'bg', 'sp'}) do if hl[a] then n = n + 1 end end \
             end print('ATTRS ' .. n)",
        );
        let attrs: usize = output
            .lines()
            .find_map(|l| l.strip_prefix("ATTRS "))
            .and_then(|n| n.trim().parse().ok())
            .unwrap_or(0);
        h.check(attrs >= 400, format!("{id}: {attrs} colour attributes set"));

        // Every documented capture, and the named plugin groups, must resolve.
        for (label, names) in [("documented captures", &groups), ("plugin groups", &plugins.to_owned())] {
            let output = probe(
                id,
                &format!(
                    "local missing = {{}} \
                     for name in ('{names}'):gmatch('%S+') do \
                       if vim.tbl_isempty(vim.api.nvim_get_hl(0, {{ name = name, link = false }})) \
                       then missing[#missing + 1] = name end \
                     end print('MISSING ' .. table.concat(missing, ' '))"
                ),
            );
            let missing = output
                .lines()
                .find_map(|l| l.strip_prefix("MISSING "))
                .unwrap_or("?")
                .trim()
                .to_owned();
            h.check(
                missing.is_empty(),
                format!(
                    "{id}: all {label} resolve{}",
                    if missing.is_empty() { String::new() } else { format!("; missing {missing}") }
                ),
            );
        }
    }

    // The checks above only mean something if an undefined group is detected.
    let output = probe(
        "acetic",
        "local hl = vim.api.nvim_get_hl(0, { name = '@no.such.capture', link = false }) \
         print('EMPTY ' .. tostring(vim.tbl_isempty(hl)))",
    );
    h.check(
        output.contains("EMPTY true"),
        "control: an undefined capture is detected",
    );

    Ok(())
}
