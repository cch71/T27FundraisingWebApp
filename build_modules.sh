#!/usr/bin/env bash
#
# Builds the dynamically loaded page modules (order/report/admin/timecard
# pages) as standalone wasm binaries and emits their JS glue + wasm into
# <dist>/modules/<name>/. The shell fetches them on demand at runtime (see
# js/src/js/module_loader.js).
#
# Invoked automatically as a Trunk post_build hook (see Trunk.toml), where
# TRUNK_STAGING_DIR and TRUNK_PROFILE are set. Can also be run standalone:
#   ./build_modules.sh [debug|release]
set -euo pipefail

cd "$(dirname "$0")"

# crate name -> public module name (must match js/src/js/module_loader.js paths)
MODULES=(
    "order_pages:orders"
    "report_pages:reports"
    "admin_pages:admin"
    "timecard_pages:timecards"
)

PROFILE="${TRUNK_PROFILE:-${1:-release}}"
if [ "$PROFILE" = "release" ]; then
    CARGO_PROFILE_FLAG="--release"
else
    CARGO_PROFILE_FLAG=""
fi

OUT_ROOT="${TRUNK_STAGING_DIR:-dist}/modules"

# wasm-bindgen CLI must exactly match the wasm-bindgen crate version.
WB_VERSION=$(sed -n '/^name = "wasm-bindgen"$/{n;s/^version = "\(.*\)"$/\1/p;}' Cargo.lock)
if [ -z "$WB_VERSION" ]; then
    echo "error: could not determine wasm-bindgen version from Cargo.lock" >&2
    exit 1
fi

find_wasm_bindgen() {
    # A PATH-installed CLI of the right version wins
    if command -v wasm-bindgen >/dev/null 2>&1; then
        if [ "$(wasm-bindgen --version | awk '{print $2}')" = "$WB_VERSION" ]; then
            command -v wasm-bindgen
            return
        fi
    fi
    # Otherwise use the copy Trunk downloaded for the main app build
    for cache_dir in \
        "$HOME/Library/Caches/dev.trunkrs.trunk" \
        "${XDG_CACHE_HOME:-$HOME/.cache}/trunk" \
        "${XDG_CACHE_HOME:-$HOME/.cache}/dev.trunkrs.trunk"; do
        candidate="$cache_dir/wasm-bindgen-$WB_VERSION/wasm-bindgen"
        if [ -x "$candidate" ]; then
            echo "$candidate"
            return
        fi
    done
    echo ""
}

WASM_BINDGEN=$(find_wasm_bindgen)
if [ -z "$WASM_BINDGEN" ]; then
    echo "error: no wasm-bindgen $WB_VERSION CLI found." >&2
    echo "       install it with: cargo install wasm-bindgen-cli --version $WB_VERSION" >&2
    exit 1
fi

for entry in "${MODULES[@]}"; do
    crate="${entry%%:*}"
    name="${entry##*:}"

    echo "building module: $name ($crate, $PROFILE)"
    # shellcheck disable=SC2086
    cargo build $CARGO_PROFILE_FLAG --quiet --target wasm32-unknown-unknown -p "$crate"

    mkdir -p "$OUT_ROOT/$name"
    "$WASM_BINDGEN" \
        --target web \
        --no-typescript \
        --out-dir "$OUT_ROOT/$name" \
        --out-name "$name" \
        "target/wasm32-unknown-unknown/$PROFILE/${crate}.wasm"
done

echo "modules written to $OUT_ROOT"
