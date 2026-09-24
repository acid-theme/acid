# fish reads both forms of the port: a .theme file applied through fish_config,
# and a conf.d script that sets the same variables globally. They must agree.
. /acid/tests/lib.sh

root=/tmp/fishconfig
mkdir -p "$root/fish/themes" "$root/share"
cp ports/fish/themes/*.theme "$root/fish/themes/"

for flavor in Acetic Citric; do
    if HOME=$root XDG_CONFIG_HOME=$root XDG_DATA_HOME=$root/share \
        fish -c "fish_config theme choose \"Acid $flavor\"" 2>/dev/null; then
        pass "$flavor: fish_config applied the theme"
    else
        fail "$flavor: fish_config rejected the theme"
    fi
done

# The two forms must set identical values for every variable.
for flavor in acetic citric; do
    title=$(printf '%s' "$flavor" | sed 's/^./\U&/')
    result=$(fish --no-config -c "
        source ports/fish/conf.d/acid-$flavor.fish
        set -l bad 0
        set -l total 0
        for line in (cat 'ports/fish/themes/Acid $title.theme' | string match -rv '^#|^\$')
            set -l parts (string split -m 1 ' ' -- \$line)
            set -l var \$parts[1]
            set -l want (string trim -- \$parts[2])
            set -l got (string join ' ' -- \$\$var)
            set total (math \$total + 1)
            if test \"\$got\" != \"\$want\"
                echo \"MISMATCH \$var conf.d='\$got' theme='\$want'\"
                set bad (math \$bad + 1)
            end
        end
        echo \"TOTAL \$total BAD \$bad\"
    " 2>&1)
    counts=$(printf '%s' "$result" | grep '^TOTAL')
    total=$(printf '%s' "$counts" | awk '{print $2}')
    bad=$(printf '%s' "$counts" | awk '{print $4}')
    declared=$(grep -c '^fish_' "ports/fish/themes/Acid $title.theme")

    if [ "${bad:-1}" != "0" ]; then
        fail "$flavor: the two forms disagree on ${bad} variable(s)"
        note "$(printf '%s' "$result" | grep MISMATCH | head -5)"
    elif [ "${total:-0}" != "$declared" ]; then
        fail "$flavor: compared ${total:-0} variables but the theme declares $declared"
    else
        pass "$flavor: both forms agree on all $total variables"
    fi
done

# fish does not validate colour values, so a bad value proves nothing. A theme
# that does not exist does fail, which is what makes the applies above meaningful.
if HOME=$root XDG_CONFIG_HOME=$root XDG_DATA_HOME=$root/share \
    fish -c 'fish_config theme choose "DoesNotExist"' >/dev/null 2>&1; then
    fail "control: a missing theme was accepted, so this test proves nothing"
else
    pass "control: fish rejects a theme that does not exist"
fi

summary
