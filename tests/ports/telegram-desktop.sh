# Telegram is a GUI client that cannot load a theme headlessly, so this checks
# the palette's structure against Telegram's own default palette, vendored at
# ports/telegram-desktop/upstream.palette. It cannot tell whether the result looks good.
. /acid/tests/lib.sh

python3 - <<'PY'
import re
import sys

passed = failed = 0

def report(ok, message):
    global passed, failed
    if ok:
        passed += 1
        print(f"    ok   {message}")
    else:
        failed += 1
        print(f"    FAIL {message}")

ENTRY = re.compile(r"^([a-zA-Z][a-zA-Z0-9]*):\s*(.+?);")

def parse(path):
    entries = {}
    for line in open(path):
        match = ENTRY.match(line)
        if match:
            entries[match.group(1)] = match.group(2).strip()
    return entries

upstream = parse("ports/telegram-desktop/upstream.palette")
report(len(upstream) > 500, f"Telegram's default palette has {len(upstream)} keys")

for flavour in ("acetic", "citric"):
    theme = parse(f"ports/telegram-desktop/themes/acid-{flavour}.tdesktop-palette")

    unknown = sorted(set(theme) - set(upstream))
    report(not unknown, f"{flavour}: every key exists upstream — {len(theme)} keys"
           + (f"; unknown: {unknown[:4]}" if unknown else ""))

    missing = sorted(set(upstream) - set(theme))
    report(not missing, f"{flavour}: no key is left out"
           + (f"; missing: {missing[:4]}" if missing else ""))

    # A value is a literal colour or another key's name, and nothing else.
    # Telegram's *source* palette also allows "value | fallback", but its theme
    # parser refuses it: "Expected ';' after each value in the color scheme".
    bad = []
    for name, value in theme.items():
        literal = r"#[0-9a-f]{3,4}|#[0-9a-f]{6}|#[0-9a-f]{8}"
        if not re.fullmatch(rf"({literal})|[a-zA-Z][a-zA-Z0-9]*", value, re.IGNORECASE):
            bad.append(f"{name}={value}")
    report(not bad, f"{flavour}: every value is a colour or a key"
           + (f"; bad: {bad[:4]}" if bad else ""))

    # A reference to a key that does not exist stops the theme loading too.
    dangling = []
    for name, value in theme.items():
        for ref in re.findall(r"(?:^|\|)\s*([a-zA-Z][a-zA-Z0-9]*)\s*$", value):
            if ref not in theme:
                dangling.append(f"{name} -> {ref}")
    report(not dangling, f"{flavour}: every reference resolves"
           + (f"; dangling: {dangling[:4]}" if dangling else ""))

    # A cycle would leave a colour undefined.
    cycles = []
    for name in theme:
        seen, current = set(), name
        while True:
            value = theme.get(current, "")
            if not re.fullmatch(r"[a-zA-Z][a-zA-Z0-9]*", value):
                break
            if value in seen:
                cycles.append(name)
                break
            seen.add(value)
            current = value
    report(not cycles, f"{flavour}: no reference cycles"
           + (f"; cycles: {cycles[:4]}" if cycles else ""))

# The grammar check is only meaningful if it refuses what Telegram refuses.
grammar = r"(#[0-9a-f]{3,4}|#[0-9a-f]{6}|#[0-9a-f]{8})|[a-zA-Z][a-zA-Z0-9]*"
report(not re.fullmatch(grammar, "#00zz00", re.IGNORECASE),
       "control: a malformed colour is refused")
report(not re.fullmatch(grammar, "#2dad2d | boxTextFgGood", re.IGNORECASE),
       "control: the source-only fallback syntax is refused")

print(f"    -- {passed} passed, {failed} failed")
sys.exit(1 if failed else 0)
PY
