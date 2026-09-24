# Two notifications at different urgencies, so the accent escalation shows.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

start_wayland "460x180" "$(base_colour base)"

mkdir -p ~/.config/mako
cp "$PWD/acid-$FLAVOUR.conf" ~/.config/mako/config
cat >> ~/.config/mako/config <<CONF
font=$FONT 11
border-size=2
border-radius=4
padding=12
margin=12
width=420
default-timeout=0
CONF

dbus-run-session -- bash -c '
    mako >/tmp/mako.log 2>&1 &
    sleep 2
    notify-send -u normal "Build finished" "cargo test: 54 passed, 0 failed"
    notify-send -u critical "Disk almost full" "3% remaining on /home"
    sleep 3
    grim "$OUT"'
