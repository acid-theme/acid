# aerc refuses to start when a styleset names an object its parser does not
# know, and its manual and bundled stylesets list objects the binary rejects.
# So the binary is asked, one object at a time, rather than the documentation.
. /acid/tests/lib.sh

require aerc

root=/tmp/cfg
mkdir -p "$root/aerc/stylesets"
printf '[ui]\nstyleset-name=probe\n' > "$root/aerc/aerc.conf"
# Left deliberately lax: aerc checks the styleset first, then complains about
# these permissions, which makes every probe exit immediately.
printf '[default]\nsource=maildir:///tmp/mail\nfrom=a <a@b.c>\n' > "$root/aerc/accounts.conf"

# Returns the rejected object, if aerc rejects one.
probe() {
    cp "$1" "$root/aerc/stylesets/probe"
    XDG_CONFIG_HOME=$root timeout 10 aerc </dev/null 2>&1 \
        | sed -n 's/.*unknown style object: //p' | head -1
}

page=$(ls /usr/share/man/man7/aerc-stylesets.7* 2>/dev/null | head -1)
[ -n "$page" ] || { fail "aerc-stylesets(7) is not installed"; summary; exit; }
zcat -f "$page" | sed -n '/^.SH STYLE OBJECTS/,/^.SH /p' \
    | grep -oE '^.fB[a-z][a-z_0-9]*.fR$' \
    | sed 's/^.fB//; s/.fR$//' | sort -u > /tmp/documented

documented=$(wc -l < /tmp/documented)
[ "$documented" -ge 30 ] || { fail "could not read aerc's manual ($documented objects)"; summary; exit; }

# Which documented objects this build of aerc actually accepts.
: > /tmp/accepted
: > /tmp/rejected
while read -r object; do
    printf '%s.fg=#ff0000\n' "$object" > /tmp/probe
    if [ -n "$(probe /tmp/probe)" ]; then
        printf '%s\n' "$object" >> /tmp/rejected
    else
        printf '%s\n' "$object" >> /tmp/accepted
    fi
done < /tmp/documented
note "aerc documents $documented objects and accepts $(wc -l < /tmp/accepted)"
[ -s /tmp/rejected ] && note "it rejects: $(tr '\n' ' ' < /tmp/rejected)"

for flavor in acetic citric; do
    styleset="ports/aerc/themes/acid-$flavor"

    # The check that matters: aerc starts with this styleset.
    bad=$(probe "$styleset")
    [ -n "$bad" ] \
        && fail "$flavor: aerc rejects the styleset at $bad" \
        || pass "$flavor: aerc accepts the styleset"

    used=$(grep -oE '^[a-z][a-z_0-9]*' "$styleset" | sort -u)
    unaccepted=$(printf '%s\n' "$used" | comm -23 - /tmp/accepted | tr '\n' ' ')
    [ -n "$(printf '%s' "$unaccepted" | tr -d ' ')" ] \
        && fail "$flavor: objects this aerc does not accept: $unaccepted" \
        || pass "$flavor: every object used is one aerc accepts"

    missing=$(printf '%s\n' "$used" | comm -13 - /tmp/accepted | tr '\n' ' ')
    [ -n "$(printf '%s' "$missing" | tr -d ' ')" ] \
        && fail "$flavor: accepted objects left unstyled: $missing" \
        || pass "$flavor: every object aerc accepts is styled"
done

# The probe is only meaningful if an invented object is refused.
printf 'nonsense_object.fg=#ff0000\n' > /tmp/probe
[ "$(probe /tmp/probe)" = "nonsense_object" ] \
    && pass "control: aerc rejects an invented object" \
    || fail "control: an invented object was accepted, so this proves nothing"

summary
