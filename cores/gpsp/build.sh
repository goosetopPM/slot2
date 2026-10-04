#!/bin/sh
# gpSP — the GBA alternative. Its libretro makefile is the repository root `Makefile`, and it
# picks its dynarec from `uname -m` when nothing says otherwise: in the x86_64 build container
# that is x86, not the H700 the core will run on. So the architecture and the dynarec are named
# here rather than inferred — for the cross builds that is true of.
#
# A native build (`CROSS_TRIPLE=`, which is what CI's host job asks for) runs on the machine
# that will load the core, and there the makefile's own autodetection is the right answer: an
# arm64 dynarec on an x86_64 runner is a core that cannot run at all. So the three arguments are
# set for an aarch64 toolchain and left off otherwise, and a triple that is neither is refused
# before anything is fetched or built.
here="$(cd "$(dirname "$0")" && pwd)"
CORE_REPO=https://github.com/libretro/gpsp
CORE_MAKEFILE=Makefile
CORE_TARGET=gpsp_libretro.so

triple="${CROSS_TRIPLE-aarch64-linux-gnu}"
case "$triple" in
"")
	CORE_MAKE_ARGS=
	;;
aarch64*)
	CORE_MAKE_ARGS="CPU_ARCH=arm64 HAVE_DYNAREC=1 MMAP_JIT_CACHE=1"
	;;
*)
	echo "gpsp: don't know how to build for '$triple'; only aarch64 cross toolchains and a native build are set up" >&2
	exit 2
	;;
esac

. "$here/../common.sh"
