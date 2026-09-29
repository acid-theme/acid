// qutebrowser's settings are typed, so its own config machinery validates both
// the names and the values. That machinery is Python, so it is asked in Python.

const QUTEBROWSER_PROBE: &str = r##"
import re, sys
from qutebrowser.config import configdata
configdata.init()

colour_types = ("QtColor", "QssColor", "ListOrValue")
settable = {name for name, opt in configdata.DATA.items()
            if name.startswith("colors.") and type(opt.typ).__name__ in colour_types}

assigned = {}
for line in open("THEME_PATH"):
    match = re.match(r'^c\.(colors\.[\w.]+) = "(.*)"$', line.strip())
    if match:
        assigned[match.group(1)] = match.group(2)

unknown = sorted(n for n in assigned if n not in configdata.DATA)
rejected = []
for name, value in assigned.items():
    option = configdata.DATA.get(name)
    if option is None:
        continue
    try:
        option.typ.to_py(value)
    except Exception as error:
        rejected.append(f"{name}={value} ({error})")
missing = sorted(settable - set(assigned))

print("ASSIGNED", len(assigned))
print("UNKNOWN", " ".join(unknown))
print("REJECTED", "; ".join(rejected[:3]))
print("MISSING", " ".join(missing))
"##;

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["qutebrowser", "python3"]) {
        return Ok(());
    }

    for flavour in h.flavours() {
        let theme = h.dist(&format!("acid-{}.py", flavour.identifier));
        // Built by replacement rather than format!, since the script is Python
        // and uses braces of its own.
        let script = QUTEBROWSER_PROBE.replace("THEME_PATH", &theme.to_string_lossy());
        let output = exec("python3", &["-c", &script]);
        let field = |name: &str| {
            output
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{name} ")))
                .unwrap_or("?")
                .trim()
                .to_owned()
        };

        let assigned = field("ASSIGNED");
        h.check(
            field("UNKNOWN").is_empty(),
            format!("{}: every setting exists — {assigned} assigned", flavour.identifier),
        );
        h.check(
            field("REJECTED").is_empty(),
            format!(
                "{}: qutebrowser accepts every value{}",
                flavour.identifier,
                if field("REJECTED").is_empty() { String::new() } else { format!("; {}", field("REJECTED")) }
            ),
        );
        h.check(
            field("MISSING").is_empty(),
            format!("{}: every colour setting is set", flavour.identifier),
        );
    }

    // The value check is only meaningful if a bad colour is refused.
    let control = exec(
        "python3",
        &["-c", r##"
from qutebrowser.config import configdata
configdata.init()
try:
    configdata.DATA["colors.statusbar.normal.bg"].typ.to_py("#00zz00")
    print("ACCEPTED")
except Exception:
    print("REFUSED")
"##],
    );
    h.check(
        control.contains("REFUSED"),
        "control: qutebrowser rejects a malformed colour",
    );

    Ok(())
}
