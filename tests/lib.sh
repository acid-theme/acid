# Shared helpers for the per-port tests. Sourced inside the container.
set -u

# The flavours to check, read from the palette so a new one is picked up here
# without editing every port's test.
FLAVOURS=$(python3 -c "
import json
print(' '.join(json.load(open('/acid/palette.json'))['flavors']))
")

_pass=0
_fail=0

pass() { _pass=$((_pass + 1)); printf '    ok   %s\n' "$*"; }
fail() { _fail=$((_fail + 1)); printf '    FAIL %s\n' "$*"; }
note() { printf '         %s\n' "$*"; }

# Assert that "$1" contains "$2".
contains() {
    case "$1" in
        *"$2"*) return 0 ;;
        *) return 1 ;;
    esac
}

# A missing program must fail loudly: several checks look for an error in a
# command's output, and absent output is not an error.
require() {
    for program in "$@"; do
        if ! command -v "$program" >/dev/null 2>&1; then
            fail "$program is not installed in the test image"
            summary
            exit 1
        fi
    done
}

summary() {
    printf '    -- %d passed, %d failed\n' "$_pass" "$_fail"
    [ "$_fail" -eq 0 ]
}
