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

# Every capture Neovim documents must resolve to something, so a capture added
# upstream shows up here rather than as an unstyled token.
captures=$(grep -oE '^@[a-z][a-z0-9._]*' /usr/share/nvim/runtime/doc/treesitter.txt \
    | sort -u | grep -v '^@none$')
count=$(printf '%s\n' "$captures" | wc -l)
if [ "$count" -lt 50 ]; then
    fail "could not read Neovim's capture list (found $count)"
else
    note "Neovim documents $count captures"
    for flavor in acetic citric; do
        undefined=$(printf '%s\n' "$captures" | nvim --headless --clean -u NONE \
            --cmd "set rtp^=/acid/ports/neovim" \
            -c "colorscheme acid-$flavor" \
            -c "lua
                local missing = {}
                for line in io.lines('/dev/stdin') do
                  local hl = vim.api.nvim_get_hl(0, { name = line, link = false })
                  if vim.tbl_isempty(hl) then missing[#missing + 1] = line end
                end
                print('MISSING ' .. table.concat(missing, ' '))
            " -c "qa!" 2>&1 | sed -n 's/^MISSING //p')
        [ -n "$undefined" ] \
            && fail "$flavor: captures with no highlight:$undefined" \
            || pass "$flavor: all $count documented captures resolve"
    done

    # The check above only means something if an undefined group is detected.
    control=$(printf '@no.such.capture\n' | nvim --headless --clean -u NONE \
        --cmd "set rtp^=/acid/ports/neovim" -c "colorscheme acid-acetic" \
        -c "lua
            local missing = {}
            for line in io.lines('/dev/stdin') do
              local hl = vim.api.nvim_get_hl(0, { name = line, link = false })
              if vim.tbl_isempty(hl) then missing[#missing + 1] = line end
            end
            print('MISSING ' .. table.concat(missing, ' '))
        " -c "qa!" 2>&1 | sed -n 's/^MISSING //p')
    contains "$control" "@no.such.capture" \
        && pass "control: an undefined capture is detected" \
        || fail "control: an undefined capture went unnoticed, so this proves nothing"
fi

# Semantic token modifiers carry the server's extra information, so they must
# resolve to a colour rather than to nothing.
for flavor in acetic citric; do
    resolved=$(nvim --headless --clean -u NONE \
        --cmd "set rtp^=/acid/ports/neovim" -c "colorscheme acid-$flavor" \
        -c "lua
            local groups = {
              '@lsp.typemod.variable.readonly',
              '@lsp.typemod.function.defaultLibrary',
              '@lsp.mod.deprecated',
            }
            local out = {}
            for _, g in ipairs(groups) do
              local hl = vim.api.nvim_get_hl(0, { name = g, link = false })
              out[#out + 1] = g .. '=' .. (hl.fg and string.format('#%06x', hl.fg)
                or (hl.strikethrough and 'strikethrough' or 'none'))
            end
            print('SEM ' .. table.concat(out, ' '))
        " -c "qa!" 2>&1 | sed -n 's/^SEM //p')
    contains "$resolved" "none" \
        && fail "$flavor: a semantic token modifier resolved to nothing — $resolved" \
        || pass "$flavor: semantic token modifiers resolve — $resolved"
done

summary
