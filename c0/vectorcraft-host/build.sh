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
# Size tuning for the guest page only (the 70 MiB cap in the C0X pin): smallest code, one unit, fat LTO, no debug info.
# Same recipe that took C0pdf from 46.4 MB to 30.4 MB on 2026-10-28; panic strategy stays upstream's.
export CARGO_PROFILE_RELEASE_OPT_LEVEL=z
export CARGO_PROFILE_RELEASE_LTO=fat
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_STRIP=debuginfo
exec smoac-slot run heavy 2 3600 -- trunk build --release
