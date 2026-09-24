# niri validates its own config. An included `layout` block merges with the
# config's, which a second one in the same file cannot — the port depends on
# that, so it is asserted here.
. /acid/tests/lib.sh

require niri

for flavor in acetic citric; do
    theme="ports/niri/themes/acid-$flavor.kdl"
    if niri validate -c "$theme" 2>&1 | grep -q "config is valid"; then
        pass "$flavor: niri validated $theme"
    else
        fail "$flavor: niri rejected $theme"
        note "$(niri validate -c "$theme" 2>&1 | grep -v DEBUG | head -6)"
    fi
done

# The port's premise: include + a local layout block coexist.
cp ports/niri/themes/acid-acetic.kdl /tmp/theme.kdl
printf 'include "/tmp/theme.kdl"\nlayout {\n    gaps 4\n    border {\n        width 0.5\n    }\n}\n' > /tmp/config.kdl
if niri validate -c /tmp/config.kdl 2>&1 | grep -q "config is valid"; then
    pass "an included layout block merges with a local one"
else
    fail "include no longer merges; the port's install instructions are wrong"
fi

# Two layout blocks in one file must still be an error, or the above proves
# nothing about merging.
printf 'layout {\n    gaps 4\n}\nlayout {\n    gaps 8\n}\n' > /tmp/dup.kdl
if niri validate -c /tmp/dup.kdl 2>&1 | grep -q "duplicate node"; then
    pass "control: two layout blocks in one file are rejected"
else
    fail "control: duplicate layout accepted, so merging is not being tested"
fi

summary
