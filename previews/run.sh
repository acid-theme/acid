#!/usr/bin/env bash
# Render a preview of each port the way its own repository will: build the
# mirror, build that port's preview image, and run its render script against it.
# Usage: previews/run.sh [port...]
set -euo pipefail

cd "$(dirname "$0")/.."

FLAVOURS=(acetic citric)

if ! command -v podman >/dev/null 2>&1; then
    echo "previews: podman is not installed" >&2
    exit 1
fi

ports=("$@")
if [ ${#ports[@]} -eq 0 ]; then
    for script in previews/ports/*.sh; do
        ports+=("$(basename "$script" .sh)")
    done
fi

python3 scripts/publish.py --out target/mirrors >/dev/null

status=0
for port in "${ports[@]}"; do
    mirror="target/mirrors/$port"
    if [ ! -d "$mirror" ]; then
        echo "previews: no such port: $port" >&2
        status=1
        continue
    fi

    image="acid-preview-$port"
    if ! podman image exists "$image"; then
        echo "previews: building $image"
        podman build -q -t "$image" -f "$mirror/preview/Containerfile" "$mirror/preview" >/dev/null
    fi

    mkdir -p "target/previews/$port"
    for flavour in "${FLAVOURS[@]}"; do
        printf '  %-10s %s ... ' "$port" "$flavour"
        if podman run --rm \
            -v "$PWD/$mirror:/repo:Z" \
            -v "$PWD/target/previews/$port:/out:Z" \
            -w /repo \
            -e "FLAVOUR=$flavour" \
            -e "OUT=/out/$flavour.png" \
            "$image" bash preview/render.sh >/tmp/preview-$port-$flavour.log 2>&1; then
            identify -format '%wx%h\n' "target/previews/$port/$flavour.png" 2>/dev/null || echo written
        else
            printf 'FAILED\n'
            tail -4 "/tmp/preview-$port-$flavour.log" | sed 's/^/      /'
            status=1
        fi
    done
done

exit "$status"
