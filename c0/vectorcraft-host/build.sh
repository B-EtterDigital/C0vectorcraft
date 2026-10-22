#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
if [ -n "${CRAFT_FONTS_DIR:-}" ] && [ ! -d "$CRAFT_FONTS_DIR" ]; then
    echo "CRAFT_FONTS_DIR is missing; building without optional craft-fonts web faces." >&2
    unset CRAFT_FONTS_DIR
fi
if [ -n "${CRAFT_FONTS_DIR:-}" ]; then
    case "$CRAFT_FONTS_DIR" in /*) ;; *) export CRAFT_FONTS_DIR="$PWD/$CRAFT_FONTS_DIR" ;; esac
fi
cd c0/vectorcraft-host
export CARGO_BUILD_JOBS=2
export BINARYEN_CORES="$CARGO_BUILD_JOBS"
export NO_COLOR=true
exec smoac-slot run heavy 2 3600 -- trunk build --release
