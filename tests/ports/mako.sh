# mako parses its config before it touches Wayland or DBus, so a failure to
# start on either means the config was read successfully.
. /acid/tests/lib.sh

require mako

parses() {
    # Succeeds when mako gets past config parsing.
    ! env -u WAYLAND_DISPLAY XDG_CONFIG_HOME="$1" timeout 5 mako 2>&1 \
        | grep -q "Failed to parse"
}

for flavor in acetic citric; do
    root=/tmp/$flavor
    mkdir -p "$root/mako"
    cp "ports/mako/themes/acid-$flavor.conf" "$root/mako/config"
    if parses "$root"; then
        pass "$flavor: mako parsed the theme"
    else
        fail "$flavor: mako rejected the theme"
    fi

    # The theme is meant to be included, with the including file's own globals
    # after it. Criteria sections must not swallow them.
    root=/tmp/$flavor-include
    mkdir -p "$root/mako"
    cp "ports/mako/themes/acid-$flavor.conf" "$root/mako/theme.conf"
    printf 'include=%s/mako/theme.conf\nmax-visible=7\ndefault-timeout=10000\n' \
        "$root" > "$root/mako/config"
    if parses "$root"; then
        pass "$flavor: globals after the include are still globals"
    else
        fail "$flavor: the theme's sections leaked into the including file"
    fi
done

mkdir -p /tmp/control/mako
printf 'background-color=#000000\nnonsense-option=1\n' > /tmp/control/mako/config
if parses /tmp/control; then
    fail "control: a bad option was accepted, so this test proves nothing"
else
    pass "control: mako rejects an unknown option"
fi

summary
