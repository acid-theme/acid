# Neovim loads the colourscheme and reports what actually resolved.
. /acid/tests/lib.sh

require nvim

for flavor in acetic citric; do
    output=$(nvim --headless --clean -u NONE \
        --cmd "set rtp^=/acid/ports/neovim" \
        -c "lua
            local ok, err = pcall(vim.cmd.colorscheme, 'acid-$flavor')
            if not ok then print('ERROR ' .. tostring(err)) end
            print('NAME ' .. tostring(vim.g.colors_name))
            local n = 0
            for _, hl in pairs(vim.api.nvim_get_hl(0, {})) do
              for _, a in ipairs({'fg', 'bg', 'sp'}) do
                if hl[a] then n = n + 1 end
              end
            end
            print('ATTRS ' .. n)
            local function fg(g)
              local h = vim.api.nvim_get_hl(0, { name = g, link = false })
              return h.fg and string.format('#%06x', h.fg) or 'none'
            end
            print('KEYWORD ' .. fg('@keyword'))
            print('LSPPROP ' .. fg('@lsp.type.property'))
        " -c "qa!" 2>&1)

    if contains "$output" "ERROR "; then
        fail "$flavor: colourscheme failed to load"
        note "$output"
        continue
    fi
    pass "$flavor: colourscheme loaded"

    contains "$output" "NAME acid-$flavor" \
        && pass "$flavor: colors_name is acid-$flavor" \
        || fail "$flavor: colors_name wrong"

    attrs=$(printf '%s' "$output" | sed -n 's/^ATTRS //p')
    if [ "${attrs:-0}" -ge 250 ]; then
        pass "$flavor: $attrs colour attributes set"
    else
        fail "$flavor: only ${attrs:-0} colour attributes set"
    fi

    # Semantic tokens must resolve through to the treesitter groups.
    keyword=$(printf '%s' "$output" | sed -n 's/^KEYWORD //p')
    lspprop=$(printf '%s' "$output" | sed -n 's/^LSPPROP //p')
    [ "$keyword" != "none" ] \
        && pass "$flavor: @keyword is $keyword" \
        || fail "$flavor: @keyword unset"
    [ "$lspprop" != "none" ] \
        && pass "$flavor: @lsp.type.property resolves to $lspprop" \
        || fail "$flavor: @lsp.type.property did not resolve"
done

summary
