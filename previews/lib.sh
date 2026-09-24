# Helpers for a port's preview script. Runs inside the port's own repository,
# where $FLAVOUR names the flavour and $OUT is the file to write. The working
# directory is the repository root, so themes are referred to by their published
# paths.
set -eu

FONT="JetBrainsMono Nerd Font"
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

mkdir -p /tmp/xdg
chmod 700 /tmp/xdg
export XDG_RUNTIME_DIR=/tmp/xdg

# Start a headless X server of the given size and point DISPLAY at it.
start_x11() {
    Xvfb :99 -screen 0 "$1x24" >/tmp/xvfb.log 2>&1 &
    export DISPLAY=:99
    export WINIT_UNIX_BACKEND=x11
    # Waiting on the socket rather than xdpyinfo keeps the image one package
    # smaller.
    for _ in $(seq 40); do
        if [ -S /tmp/.X11-unix/X99 ]; then
            sleep 0.5
            return 0
        fi
        sleep 0.2
    done
    echo "preview: Xvfb did not start" >&2
    tail -3 /tmp/xvfb.log >&2
    return 1
}

# Start a headless wlroots compositor of the given size, with no decoration so a
# client fills the output. A second argument paints the backdrop.
start_wayland() {
    printf 'output HEADLESS-1 resolution %s %s\ndefault_border none\ngaps inner 0\n' \
        "$1" "${2:+bg $2 solid_color}" > /tmp/sway.conf
    WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 sway -c /tmp/sway.conf \
        >/tmp/sway.log 2>&1 &
    for _ in $(seq 30); do
        if [ -e /tmp/xdg/wayland-1 ]; then
            export WAYLAND_DISPLAY=wayland-1
            sleep 1
            return 0
        fi
        sleep 0.2
    done
    echo "preview: sway did not start" >&2
    tail -3 /tmp/sway.log >&2
    return 1
}

# Run a command in Alacritty under X11, themed by this port's own Alacritty
# theme when it has one and by a bundled copy otherwise.
in_terminal() {
    local size=$1 columns=$2 lines=$3
    shift 3
    start_x11 "$size"
    cat > /tmp/alacritty.toml <<TOML
general.import = ["$ALACRITTY_THEME"]
[font]
size = ${TERMINAL_FONT_SIZE:-11}
[font.normal]
family = "$FONT"
[window]
padding = { x = 14, y = 12 }
dimensions = { columns = $columns, lines = $lines }
[cursor]
style = { blinking = "Off" }
TOML
    alacritty --config-file /tmp/alacritty.toml -e "$@" >/tmp/alacritty.log 2>&1 &
    sleep 5
    WINDOW=$(xdotool search --class Alacritty | head -1)
    export WINDOW
}

# Captures the terminal window rather than the whole root, so the image is the
# theme and nothing else.
capture_x11() { import -window "${WINDOW:-root}" "$1"; }
capture_wayland() { grim "$1"; }

# A colour from the palette shipped alongside the preview scripts.
base_colour() {
    python3 -c "import json;print(json.load(open('$HERE/palette.json'))['flavors']['$FLAVOUR']['colors']['$1']['hex'])"
}
