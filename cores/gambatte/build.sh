#!/bin/sh
# Gambatte — the Game Boy / Game Boy Color alternative. The repository's root `Makefile` only
# includes `Makefile.libretro`, which is the real build file, so the shared steps' default
# makefile is already the right one and nothing else needs saying.
here="$(cd "$(dirname "$0")" && pwd)"
CORE_REPO=https://github.com/libretro/gambatte-libretro
CORE_TARGET=gambatte_libretro.so
. "$here/../common.sh"
