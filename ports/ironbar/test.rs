// Ironbar is GTK4, so the theme is checked by GTK4's own CSS parser, reached
// through Python's GTK bindings. GTK4 reports deprecations through the same
// parsing-error signal, so a clean parse also means nothing in the theme is
// deprecated.

const IRONBAR_PROBE: &str = r##"
import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

theme = open("THEME_PATH").read()
usage = """USAGE_CSS"""

def parse(text):
    errors = []
    provider = Gtk.CssProvider()
    provider.connect("parsing-error", lambda p, s, e: errors.append(e.message))
    provider.load_from_string(text)
    return errors

print("GTK", Gtk.get_major_version())
print("THEME", "; ".join(parse(theme)))
print("USAGE", "; ".join(parse(theme + chr(10) + usage)))
print("UNDEFINED", "; ".join(parse("label { color: @no_such_colour; }")))
print("MALFORMED", "; ".join(parse("@define-color c notacolour;")))
"##;

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["python3"]) {
        return Ok(());
    }

    let roles: Vec<&str> = acid_palette::ACETIC
        .colors
        .iter()
        .map(|c| c.identifier)
        .collect();
    let usage: String = roles
        .iter()
        .enumerate()
        .map(|(i, role)| format!("#m{i} {{ color: @{role}; }}"))
        .collect::<Vec<_>>()
        .join("\n");

    for flavour in h.flavours() {
        let theme = h.dist(&format!("acid-{}.css", flavour.identifier));
        let script = IRONBAR_PROBE
            .replace("THEME_PATH", &theme.to_string_lossy())
            .replace("USAGE_CSS", &usage);
        let output = exec("python3", &["-c", &script]);
        let field = |name: &str| {
            output
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{name} ")))
                .unwrap_or("?")
                .trim()
                .to_owned()
        };

        h.check(
            field("GTK") == "4",
            format!("{}: the probe ran under GTK 4", flavour.identifier),
        );
        h.check(
            field("THEME").is_empty(),
            format!("{}: GTK 4 parsed the theme", flavour.identifier),
        );
        h.check(
            field("USAGE").is_empty(),
            format!(
                "{}: all {} roles resolve in a stylesheet",
                flavour.identifier,
                roles.len()
            ),
        );

        if flavour.identifier == "acetic" {
            // GTK accepts an undefined colour name silently, so the role check
            // above is what catches a missing role — not GTK. Pinned here
            // because the install instructions depend on knowing which way
            // round it is.
            h.check(
                field("UNDEFINED").is_empty(),
                "an undefined colour name is accepted silently, as documented",
            );
            h.check(
                !field("MALFORMED").is_empty(),
                "control: GTK rejects a malformed colour value",
            );
        }
    }

    Ok(())
}
