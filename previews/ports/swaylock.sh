# The lock screen, with a few keystrokes sent so the ring shows its keypress
# segments rather than only the idle state.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

start_wayland "620x360"

swaylock -C "$PWD/acid-$FLAVOUR.conf" \
    --indicator-idle-visible --indicator-radius 70 --indicator-thickness 9 \
    --font "$FONT" >/tmp/swaylock.log 2>&1 &
sleep 5
wtype "acid" 2>/dev/null || true
sleep 1
capture_wayland "$OUT"
