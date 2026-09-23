#!/bin/sh
# Genesis Plus GX — Mega Drive *and* Master System. One core covers both shelves, which is
# why the registry names it for MD and SMS alike.
here="$(cd "$(dirname "$0")" && pwd)"
CORE_REPO=https://github.com/libretro/Genesis-Plus-GX
CORE_TARGET=genesis_plus_gx_libretro.so
. "$here/../common.sh"
