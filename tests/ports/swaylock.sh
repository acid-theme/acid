# swaylock exits rather than locking when it meets a key it does not know, so
# every key is checked against its own --help output.
. /acid/tests/lib.sh

require swaylock

swaylock --help 2>&1 | grep -oE '^\s+--[a-z-]+|^\s+-[A-Za-z], --[a-z-]+' \
    | grep -oE -- '--[a-z-]+' | sed 's/^--//' | sort -u > /tmp/options

known=$(wc -l < /tmp/options)
if [ "$known" -lt 20 ]; then
    fail "could not read swaylock's option list (found $known)"
    summary
    exit
fi
note "swaylock advertises $known options"

for flavor in acetic citric; do
    theme="ports/swaylock/themes/acid-$flavor.conf"
    unknown=""
    malformed=""
    count=0

    while IFS= read -r line; do
        case "$line" in ''|'#'*) continue ;; esac
        key=${line%%=*}
        value=${line#*=}
        count=$((count + 1))
        grep -qx "$key" /tmp/options || unknown="$unknown $key"
        printf '%s' "$value" | grep -qxE '[0-9a-f]{6}([0-9a-f]{2})?' \
            || malformed="$malformed $key=$value"
    done < "$theme"

    [ -n "$unknown" ] \
        && fail "$flavor: keys swaylock does not know:$unknown" \
        || pass "$flavor: all $count keys are known to swaylock"

    [ -n "$malformed" ] \
        && fail "$flavor: values outside <rrggbb[aa]>:$malformed" \
        || pass "$flavor: all $count values are bare 6 or 8 digit hex"

    # Every colour option swaylock offers should be set, or it falls back to its
    # own light grey.
    missing=""
    for option in $(grep -E -- '(^color$|-color$)' /tmp/options); do
        grep -q "^$option=" "$theme" || missing="$missing $option"
    done
    [ -n "$missing" ] \
        && fail "$flavor: colour options left unset:$missing" \
        || pass "$flavor: every colour option swaylock offers is set"
done

# The key check is only meaningful if an invented key would be caught.
grep -qx "nonsense-color" /tmp/options \
    && fail "control: an invented key was found in the option list" \
    || pass "control: an invented key is absent from the option list"

summary
