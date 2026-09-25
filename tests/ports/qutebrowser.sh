# qutebrowser's settings are typed, so its own config machinery validates both
# the names and the values rather than a pattern written here.
. /acid/tests/lib.sh

require qutebrowser python3

python3 - <<'PY'
import re
import sys
from qutebrowser.config import configdata

configdata.init()
colour_types = ("QtColor", "QssColor", "ListOrValue")
settable = {
    name for name, opt in configdata.DATA.items()
    if name.startswith("colors.") and type(opt.typ).__name__ in colour_types
}

passed = failed = 0

def report(ok, message):
    global passed, failed
    if ok:
        passed += 1
        print(f"    ok   {message}")
    else:
        failed += 1
        print(f"    FAIL {message}")

for flavour in ("acetic", "citric"):
    path = f"ports/qutebrowser/themes/acid-{flavour}.py"
    assigned = {}
    for line in open(path):
        match = re.match(r'^c\.(colors\.[\w.]+) = "(.*)"$', line.strip())
        if match:
            assigned[match.group(1)] = match.group(2)

    unknown = sorted(n for n in assigned if n not in configdata.DATA)
    report(not unknown, f"{flavour}: every setting exists — {len(assigned)} assigned"
           + (f"; unknown: {unknown}" if unknown else ""))

    # qutebrowser parses each value with the option's own type.
    rejected = []
    for name, value in assigned.items():
        option = configdata.DATA.get(name)
        if option is None:
            continue
        try:
            option.typ.to_py(value)
        except Exception as error:
            rejected.append(f"{name}={value} ({error})")
    report(not rejected, f"{flavour}: qutebrowser accepts every value"
           + (f"; rejected: {rejected[:3]}" if rejected else ""))

    missing = sorted(settable - set(assigned))
    report(not missing, f"{flavour}: every colour setting is set"
           + (f"; missing: {missing}" if missing else ""))

# The value check is only meaningful if a bad colour is refused.
try:
    configdata.DATA["colors.statusbar.normal.bg"].typ.to_py("#00zz00")
    report(False, "control: a malformed colour was accepted, so this proves nothing")
except Exception:
    report(True, "control: qutebrowser rejects a malformed colour")

print(f"    -- {passed} passed, {failed} failed")
sys.exit(1 if failed else 0)
PY
