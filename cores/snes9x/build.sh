#!/bin/sh
# Snes9x — the SNES core. Its libretro makefile lives in a subdirectory rather than at the
# root, and it is C++, so the shared script hands it CXX as well.
here="$(cd "$(dirname "$0")" && pwd)"
CORE_REPO=https://github.com/libretro/snes9x
CORE_MAKE_DIR=libretro
CORE_MAKEFILE=Makefile
CORE_TARGET=snes9x_libretro.so
. "$here/../common.sh"
