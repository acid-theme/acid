# A command line typed into fish, so the colours are the ones fish applies as
# text is entered rather than a printed imitation.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

cat > /tmp/fishrc.fish <<FISH
source $PWD/conf.d/acid-$FLAVOUR.fish
set -g fish_greeting ""
function fish_prompt
    set_color \$fish_color_cwd
    echo -n "~/sources/acid"
    set_color normal
    echo -n " > "
end
FISH

ALACRITTY_THEME="$HERE/alacritty-$FLAVOUR.toml"
in_terminal "820x400" 84 6 fish -C 'source /tmp/fishrc.fish'

# There is no window manager under Xvfb, so focus has to be set before typing.
xdotool windowfocus --sync "$WINDOW"
xdotool type --delay 40 'echo "acid" | string upper'
xdotool key Return
sleep 1
# Left uncommitted: this is the line whose highlighting is on display.
xdotool type --delay 40 'grep -rn --color=auto "fish_color" conf.d/ > /tmp/hits.txt'
sleep 1
capture_x11 "$OUT"
