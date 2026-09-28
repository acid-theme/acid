# aerc has no way to validate a styleset, so the styleset is checked against the
# object and attribute lists in aerc's own manual.
. /acid/tests/lib.sh

require aerc

page=$(ls /usr/share/man/man7/aerc-stylesets.7* 2>/dev/null | head -1)
[ -n "$page" ] || { fail "aerc-stylesets(7) is not installed"; summary; exit; }

# The manual is roff: a table cell is a bold run, and the attributes are
# documented as <object>.name. The backslash is matched as `.` to keep the
# pattern readable through two layers of quoting.
zcat -f "$page" | sed -n '/^.SH STYLE OBJECTS/,/^.SH /p' \
    | grep -oE '^.fB[a-z][a-z_0-9]*.fR$' \
    | sed 's/^.fB//; s/.fR$//' | sort -u > /tmp/objects
zcat -f "$page" | sed -n '/^.SH ATTRIBUTES/,/^.SH /p' \
    | grep -oE 'fB[a-z]+.fR = ' | sed 's/^fB//; s/.fR = $//' | sort -u > /tmp/attrs

objects=$(wc -l < /tmp/objects)
attrs=$(wc -l < /tmp/attrs)
if [ "$objects" -lt 30 ] || [ "$attrs" -lt 5 ]; then
    fail "could not read aerc's manual (objects=$objects attrs=$attrs)"
    summary
    exit
fi
note "aerc documents $objects style objects and $attrs attributes"

for flavor in acetic citric; do
    styleset="ports/aerc/themes/acid-$flavor"
    bad_object=""
    bad_attr=""
    bad_value=""
    count=0

    while IFS= read -r line; do
        case "$line" in ''|'#'*) continue ;; esac
        key=${line%%=*}
        value=${line#*=}
        count=$((count + 1))

        object=${key%%.*}
        attr=${key##*.}
        [ "$object" = "*" ] || grep -qx "$object" /tmp/objects \
            || bad_object="$bad_object $object"
        grep -qx "$attr" /tmp/attrs || bad_attr="$bad_attr $key"

        case "$attr" in
            fg|bg)
                printf '%s' "$value" | grep -qxE '#[0-9a-f]{6}' \
                    || bad_value="$bad_value $key=$value" ;;
            *)
                printf '%s' "$value" | grep -qxE 'true|false|toggle' \
                    || bad_value="$bad_value $key=$value" ;;
        esac
    done < "$styleset"

    [ -n "$bad_object" ] \
        && fail "$flavor: objects aerc does not define:$bad_object" \
        || pass "$flavor: all $count keys name an object aerc defines"
    [ -n "$bad_attr" ] \
        && fail "$flavor: attributes aerc does not define:$bad_attr" \
        || pass "$flavor: all $count keys name an attribute aerc defines"
    [ -n "$bad_value" ] \
        && fail "$flavor: values of the wrong shape:$bad_value" \
        || pass "$flavor: colours are #rrggbb and flags are true, false or toggle"

    missing=""
    while read -r object; do
        grep -q "^$object\." "$styleset" || missing="$missing $object"
    done < /tmp/objects
    [ -n "$missing" ] \
        && fail "$flavor: objects left unstyled:$missing" \
        || pass "$flavor: every documented object is styled"
done

grep -qx "nonsense_object" /tmp/objects \
    && fail "control: an invented object was found in aerc's list" \
    || pass "control: an invented object is absent from aerc's list"

summary
