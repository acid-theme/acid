# btop accepts anything: a malformed colour, an unknown key and a missing themes
# directory all pass without a word. So the theme is checked against the key set
# btop itself ships, rather than by asking btop.
. /acid/tests/lib.sh

require btop

# Every key any bundled theme uses, which is btop's vocabulary.
grep -hoE '^theme\[[a-z_]+\]' /usr/share/btop/themes/*.theme \
    | sed 's/.*\[//;s/\]//' | sort -u > /tmp/known
known=$(wc -l < /tmp/known)
if [ "$known" -lt 20 ]; then
    fail "could not read btop's key set (found $known)"
    summary
    exit
fi
note "btop ships $known theme keys"

for flavor in acetic citric; do
    theme="ports/btop/themes/acid-$flavor.theme"
    unknown=""
    malformed=""
    count=0

    while IFS= read -r line; do
        case "$line" in ''|'#'*) continue ;; esac
        key=${line#theme[}
        key=${key%%]*}
        value=${line#*=}
        value=$(printf '%s' "$value" | tr -d '"')
        count=$((count + 1))
        grep -qx "$key" /tmp/known || unknown="$unknown $key"
        printf '%s' "$value" | grep -qxE '#[0-9a-f]{6}' \
            || malformed="$malformed $key=$value"
    done < "$theme"

    [ -n "$unknown" ] \
        && fail "$flavor: keys btop does not use:$unknown" \
        || pass "$flavor: all $count keys are ones btop uses"

    [ -n "$malformed" ] \
        && fail "$flavor: values that are not #rrggbb:$malformed" \
        || pass "$flavor: all $count values are #rrggbb"

    missing=""
    while read -r key; do
        grep -q "^theme\[$key\]=" "$theme" || missing="$missing $key"
    done < /tmp/known
    [ -n "$missing" ] \
        && fail "$flavor: keys left unset:$missing" \
        || pass "$flavor: every key btop ships is set"
done

# The key check is only meaningful if an invented key would be caught.
grep -qx "nonsense_key" /tmp/known \
    && fail "control: an invented key was found in btop's set" \
    || pass "control: an invented key is absent from btop's set"

# Pinned, because it is the reason the checks above read files rather than ask
# btop: a theme with a malformed colour still starts without complaint.
mkdir -p /tmp/broken
printf 'theme[main_bg]="#00zz00"\n' > /tmp/broken/broken.theme
output=$(timeout 4 script -qec "btop --themes-dir /tmp/broken" /dev/null </dev/null 2>&1 | tr -d '\r')
if contains "$output" "ERROR"; then
    fail "btop now reports theme errors; this test can ask btop instead"
else
    pass "btop accepts a malformed theme silently, as documented"
fi

summary
