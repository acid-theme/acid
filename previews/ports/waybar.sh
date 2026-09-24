# The bar as Waybar draws it, on an output exactly its own height.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

start_wayland "900x34"

mkdir -p ~/.config/waybar
cp "$PWD/acid-$FLAVOUR.css" ~/.config/waybar/theme.css
cat > ~/.config/waybar/config <<'JSON'
{
  "layer": "top",
  "height": 34,
  "modules-left": ["custom/workspaces", "custom/window"],
  "modules-right": ["cpu", "memory", "battery", "clock"],
  "custom/workspaces": { "format": "● ○ ○" },
  "custom/window": { "format": "nvim  acid.lua.tera" },
  "cpu": { "format": "󰍛 12%" },
  "memory": { "format": "󰘚 31%" },
  "battery": { "format": "󰁹 87%" },
  "clock": { "format": "󰥔 24 Sep 09:41" }
}
JSON
cat > ~/.config/waybar/style.css <<CSS
@import "theme.css";
* { font-family: "$FONT"; font-size: 13px; }
window#waybar { background-color: @base; color: @text; }
#custom-workspaces { color: @aqua; padding: 0 12px; }
#custom-window { color: @overlay2; padding: 0 8px; }
#cpu { color: @blue; padding: 0 10px; }
#memory { color: @purple; padding: 0 10px; }
#battery { color: @green; padding: 0 10px; }
#clock { color: @yellow; padding: 0 14px; }
CSS

dbus-run-session -- bash -c 'waybar >/tmp/waybar.log 2>&1 & sleep 5; grim "$OUT"'
