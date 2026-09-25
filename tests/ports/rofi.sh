# rofi parses the theme itself. `-dump-theme` reports a parse failure on stderr
# but still exits 0, so the output is what gets checked.
. /acid/tests/lib.sh

require rofi

parse_errors() {
    env -u DISPLAY -u WAYLAND_DISPLAY rofi -no-config -theme "$1" -dump-theme 2>&1 >/dev/null
}

roles="red orange yellow green aqua blue purple text subtext1 subtext0
       overlay2 overlay1 overlay0 surface2 surface1 surface0 base mantle crust"

for flavor in acetic citric; do
    theme="ports/rofi/themes/acid-$flavor.rasi"
    if contains "$(parse_errors "$theme")" "Failed to parse"; then
        fail "$flavor: rofi rejected $theme"
    else
        pass "$flavor: rofi parsed the theme"
    fi

    # rofi accepts an undefined name silently, so it cannot report a missing
    # role. The file is checked directly instead.
    missing=""
    for role in $roles; do
        grep -qE "^\s+$role:" "$theme" || missing="$missing $role"
    done
    [ -n "$missing" ] \
        && fail "$flavor: roles not defined:$missing" \
        || pass "$flavor: all 19 roles defined"

    # A theme that imports this one and uses every role must still parse.
    probe=/tmp/probe-$flavor.rasi
    : > "$probe"
    printf '@import "%s/%s"\n' "$PWD" "${theme%.rasi}" >> "$probe"
    i=0
    for role in $roles; do
        printf 'element-%s { background-color: @%s; }\n' "$i" "$role" >> "$probe"
        i=$((i + 1))
    done
    if contains "$(parse_errors "$probe")" "Failed to parse"; then
        fail "$flavor: a theme importing it and using every role was rejected"
    else
        pass "$flavor: imports and resolves in another theme"
    fi
done

# Documented behaviour: an unknown name is not an error, which is why the role
# check above reads the file rather than asking rofi. The name avoids an
# underscore, which is a syntax error in rasi and would test the wrong thing.
printf 'window { background-color: @nosuchrole; }\n' > /tmp/undefined.rasi
if contains "$(parse_errors /tmp/undefined.rasi)" "Failed to parse"; then
    fail "an undefined variable is now an error; the role check can be simplified"
else
    pass "an undefined variable is accepted silently, as documented"
fi

# The control proves the parse check can fail.
printf '* {\n  base: #00zz00;\n' > /tmp/broken.rasi
if contains "$(parse_errors /tmp/broken.rasi)" "Failed to parse"; then
    pass "control: rofi rejects a broken theme"
else
    fail "control: a broken theme was accepted, so this test proves nothing"
fi

summary
