# rofi drawing its own menu. The layout follows a conventional rofi theme —
# prompt, filter counters, dashed separator, alternating rows — so the preview
# shows the roles doing real work rather than a bare list.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

start_x11 "620x420"

cat > /tmp/preview.rasi <<RASI
@import "$PWD/acid-$FLAVOUR"

* {
    font:             "$FONT 11";
    background-color: transparent;
    text-color:       @text;
    spacing:          2;
}

window {
    background-color: @base;
    border:           1px;
    border-color:     @surface2;
    padding:          8px;
    width:            560px;
}

mainbox {
    padding: 0;
    border:  0;
}

inputbar {
    padding:  4px;
    spacing:  4px;
    children: [ prompt, textbox-prompt-colon, entry, num-filtered-rows, textbox-num-sep, num-rows ];
}

prompt {
    text-color: @aqua;
}

textbox-prompt-colon {
    expand:     false;
    str:        ":";
    margin:     0px 0.3em 0px 0px;
    text-color: @overlay1;
}

entry {
    placeholder:       "Type to filter";
    placeholder-color: @overlay1;
}

num-filtered-rows,
num-rows {
    expand:     false;
    text-color: @overlay1;
}

textbox-num-sep {
    expand:     false;
    str:        "/";
    text-color: @overlay1;
}

listview {
    padding:      4px 0px 0px;
    border:       2px dash 0px 0px;
    border-color: @surface1;
    spacing:      4px;
    scrollbar:    true;
    lines:        6;
    fixed-height: 0;
}

scrollbar {
    width:        4px;
    handle-width: 8px;
    handle-color: @overlay0;
}

element {
    padding: 4px;
    spacing: 4px;
    border:  0;
}

element normal.normal    { text-color: @text; }
element normal.active    { text-color: @blue; }
element normal.urgent    { text-color: @red; }

element alternate.normal { background-color: @mantle; text-color: @text; }
element alternate.active { background-color: @mantle; text-color: @blue; }
element alternate.urgent { background-color: @mantle; text-color: @red; }

element selected.normal  { background-color: @text; text-color: @mantle; }
element selected.active  { background-color: @blue; text-color: @base; }
element selected.urgent  { background-color: @red; text-color: @base; }

element-text {
    background-color: transparent;
    text-color:       inherit;
}
RASI

# -a marks a row active, -u marks one urgent, so both states are on display.
printf 'alacritty\nnvim ~/sources/acid\nfirefox\nwaybar\nsystemctl --user restart mako\nswaylock -f\n' \
    | rofi -dmenu -a 2 -u 5 -theme /tmp/preview.rasi -p "run" >/dev/null 2>&1 &
sleep 4
WINDOW=$(xdotool search --class rofi | tail -1)
export WINDOW
capture_x11 "$OUT"
