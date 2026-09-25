# btop running for real, so the meters, gradients and box outlines are its own
# rendering rather than a mock-up.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

mkdir -p ~/.config/btop/themes
cp "$PWD/acid-$FLAVOUR.theme" ~/.config/btop/themes/
cat > ~/.config/btop/btop.conf <<CONF
color_theme = "acid-$FLAVOUR"
theme_background = true
truecolor = True
vim_keys = False
rounded_corners = True
graph_symbol = "braille"
shown_boxes = "cpu mem net proc"
update_ms = 500
proc_sorting = "cpu lazy"
CONF

ALACRITTY_THEME="$HERE/alacritty-$FLAVOUR.toml"
TERMINAL_FONT_SIZE=9
in_terminal "1100x700" 130 40 btop
# btop needs a moment to fill its graphs with something other than zeroes.
sleep 6
capture_x11 "$OUT"
