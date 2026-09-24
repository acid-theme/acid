# A source file open in Neovim, with the line number column and cursor line
# visible so the fill and overlay roles show. Lua, because Neovim bundles that
# parser — the port is mostly treesitter groups, so the preview must use them.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

ALACRITTY_THEME="$HERE/alacritty-$FLAVOUR.toml"
in_terminal "760x520" 84 26 nvim \
    --cmd "set rtp^=$PWD" \
    -c "colorscheme acid-$FLAVOUR" \
    -c "set number cursorline signcolumn=yes laststatus=2" \
    -c "lua pcall(vim.treesitter.start)" \
    -c "normal 11G" \
    "$HERE/sample.lua"
capture_x11 "$OUT"
