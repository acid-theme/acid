# niri nested in a headless compositor, with two windows so the active and
# inactive border colours are both visible. The config is the documented one:
# the theme included, the geometry local.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

start_wayland "900x420"

cat > /tmp/alacritty.toml <<TOML
general.import = ["$HERE/alacritty-$FLAVOUR.toml"]
[font]
size = 8
[font.normal]
family = "$FONT"
TOML

cat > /tmp/niri.kdl <<KDL
include "$PWD/acid-$FLAVOUR.kdl"

layout {
    gaps 10
    border {
        width 3
    }
}

prefer-no-csd

spawn-at-startup "alacritty" "--config-file" "/tmp/alacritty.toml" "-e" "nvim" "-c" "set number cursorline" "-c" "lua pcall(vim.treesitter.start)" "$HERE/sample.lua"
spawn-at-startup "alacritty" "--config-file" "/tmp/alacritty.toml" "-e" "bash" "-c" "bash $HERE/ansi.sh; sleep 60"

hotkey-overlay {
    skip-at-startup
}
KDL

niri -c /tmp/niri.kdl >/tmp/niri.log 2>&1 &
sleep 8
capture_wayland "$OUT"
