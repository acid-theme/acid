# rofi drawing its own menu, with the roles mapped onto its widgets the way the
# install instructions do it.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

start_x11 "620x400"

cat > /tmp/preview.rasi <<RASI
@import "$PWD/acid-$FLAVOUR"

* {
    background-color: @base;
    text-color:       @text;
    font:             "$FONT 11";
}

window {
    border:       2px;
    border-color: @surface2;
    padding:      12px;
    width:        560px;
}

inputbar {
    background-color: @mantle;
    text-color:       @text;
    padding:          8px;
    children:         [ prompt, entry ];
}

prompt {
    text-color: @aqua;
    padding:    0 8px 0 0;
}

listview {
    lines:   6;
    padding: 8px 0 0 0;
}

element {
    padding: 6px 8px;
}

element selected.normal {
    background-color: @surface1;
    text-color:       @text;
}

element.active {
    text-color: @aqua;
}

element.urgent {
    text-color: @red;
}
RASI

printf 'alacritty\nnvim ~/sources/acid\nfirefox\nwaybar --reload\nsystemctl --user restart mako\nswaylock -f\n' \
    | rofi -dmenu -theme /tmp/preview.rasi -p "run" >/dev/null 2>&1 &
sleep 4
WINDOW=$(xdotool search --class rofi | tail -1)
export WINDOW
capture_x11 "$OUT"
