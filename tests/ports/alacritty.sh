# Alacritty parses the theme itself. `migrate --dry-run` reports a parse error in
# its output but still exits 0, so the output is what gets checked — and a
# deliberately broken file proves the check can fail.
. /acid/tests/lib.sh

for flavor in acetic citric; do
    theme="ports/alacritty/themes/acid-$flavor.toml"
    output=$(alacritty migrate --dry-run -c "$theme" 2>&1)
    if contains "$output" "migration failed"; then
        fail "$flavor: alacritty rejected $theme"
        note "$output"
    else
        pass "$flavor: alacritty parsed the theme"
    fi

    for section in colors.primary colors.normal colors.bright colors.dim; do
        if contains "$output" "[$section]"; then
            pass "$flavor: [$section] present"
        else
            fail "$flavor: [$section] missing"
        fi
    done
done

printf '[colors.primary]\nbackground = \n' > /tmp/broken.toml
if contains "$(alacritty migrate --dry-run -c /tmp/broken.toml 2>&1)" "migration failed"; then
    pass "control: alacritty rejects a broken theme"
else
    fail "control: broken theme was accepted, so this test proves nothing"
fi

summary
