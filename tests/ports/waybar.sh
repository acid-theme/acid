# GTK parses the theme, so `@define-color` resolution is checked by the same
# engine Waybar uses rather than by a regular expression.
. /acid/tests/lib.sh

python3 - <<'PY'
import sys, pathlib
import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, GLib

ROLES = ["base", "mantle", "crust", "surface0", "surface1", "surface2",
         "overlay0", "overlay1", "overlay2", "subtext0", "subtext1", "text",
         "red", "orange", "yellow", "green", "aqua", "blue", "purple"]

passed = failed = 0

def report(ok, message):
    global passed, failed
    if ok:
        passed += 1
        print(f"    ok   {message}")
    else:
        failed += 1
        print(f"    FAIL {message}")

def parse(text):
    """Return the parse errors GTK reports for a stylesheet."""
    errors = []
    provider = Gtk.CssProvider()
    provider.connect("parsing-error",
                     lambda p, section, error: errors.append(error.message))
    try:
        provider.load_from_data(text.encode())
    except GLib.Error as error:
        errors.append(error.message)
    return errors

for flavor in ("acetic", "citric"):
    theme = pathlib.Path(f"ports/waybar/themes/acid-{flavor}.css").read_text()
    report(not parse(theme), f"{flavor}: GTK parsed the theme")

    # A stylesheet that uses every role must resolve against the theme alone.
    usage = "\n".join(f"#m{i} {{ color: @{role}; }}" for i, role in enumerate(ROLES))
    errors = parse(theme + "\n" + usage)
    report(not errors, f"{flavor}: all {len(ROLES)} roles resolve in a stylesheet")
    if errors:
        print("         " + errors[0])

# GTK accepts an undefined colour name silently, so the role check above is what
# catches a missing role — not GTK. Pinned here because the install instructions
# depend on knowing which way round it is.
report(not parse("window { color: @no_such_colour; }"),
       "an undefined colour name is accepted silently, as documented")

# The control therefore uses something GTK does reject.
report(bool(parse("@define-color c notacolour;")),
       "control: GTK rejects a malformed colour value")

print(f"    -- {passed} passed, {failed} failed")
sys.exit(1 if failed else 0)
PY
