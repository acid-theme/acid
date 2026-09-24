# The ANSI slots and a syntax-shaped sample, rendered by Alacritty itself.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

ALACRITTY_THEME="$PWD/acid-$FLAVOUR.toml"
in_terminal "760x520" 84 26 bash -c "bash $HERE/ansi.sh; sleep 30"
capture_x11 "$OUT"
