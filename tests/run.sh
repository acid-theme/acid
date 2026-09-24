#!/usr/bin/env bash
# Run each port's tests in a container, against the tool that will read the
# theme. Usage: tests/run.sh [--build] [port...]
set -euo pipefail

cd "$(dirname "$0")/.."

IMAGE=acid-tests

if ! command -v podman >/dev/null 2>&1; then
    echo "tests: podman is not installed" >&2
    exit 1
fi

build=false
ports=()
for arg in "$@"; do
    case "$arg" in
        --build) build=true ;;
        -*) echo "tests: unknown option $arg" >&2; exit 1 ;;
        *) ports+=("$arg") ;;
    esac
done

if [ "$build" = true ] || ! podman image exists "$IMAGE"; then
    echo "tests: building $IMAGE"
    podman build -t "$IMAGE" -f tests/Containerfile tests/
fi

if [ ${#ports[@]} -eq 0 ]; then
    for script in tests/ports/*.sh; do
        ports+=("$(basename "$script" .sh)")
    done
fi

status=0
for port in "${ports[@]}"; do
    script="tests/ports/$port.sh"
    if [ ! -f "$script" ]; then
        echo "tests: no such port test: $port" >&2
        status=1
        continue
    fi
    echo "== $port"
    if ! podman run --rm -v "$PWD:/acid:ro,Z" "$IMAGE" bash "$script"; then
        status=1
    fi
done

if [ "$status" -eq 0 ]; then
    echo "tests: all ports passed"
else
    echo "tests: failures above" >&2
fi
exit "$status"
