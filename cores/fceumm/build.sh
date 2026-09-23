#!/bin/sh
# FCEUmm — the NES core. See cores/common.sh for the interface and the shared steps.
here="$(cd "$(dirname "$0")" && pwd)"
CORE_REPO=https://github.com/libretro/libretro-fceumm
CORE_TARGET=fceumm_libretro.so
. "$here/../common.sh"
