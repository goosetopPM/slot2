#!/bin/sh
# Builds mGBA's libretro core from libretro/mgba at the commit in ./commit, with every patch
# beside this script applied, cross-compiled for the H700 (aarch64, glibc 2.31) inside
# build/cross.Dockerfile. Ported from slot's cores/mgba/build.sh (MIT, Brandon T. Kowalski);
# the difference is that slot built natively in an arm64 box and this cross-compiles from
# x86_64, so cmake is handed a toolchain instead of trusting uname.
#
#   build.sh stamp                      print what a build would record (.meta)
#   build.sh build WORKDIR OUT          checkout, patch, build into WORKDIR, write OUT and OUT.meta
#
# OUT.meta is how the wrapper tells a stale core from a current one: compared against `stamp`,
# a moved pin, changed patch or flag rebuilds. A buildbot core (no .meta) never passes for it.
#
# Environment:
#   CROSS_TRIPLE   toolchain prefix, default aarch64-linux-gnu. Set empty to build natively.
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
commit="$(tr -d ' \n\r' <"$here/commit")"
triple="${CROSS_TRIPLE-aarch64-linux-gnu}"

# Link-time optimisation for the device core. Measured on slot: 3-5% off every frame.
# -mcpu=cortex-a53 gave most of that back and is deliberately absent. Nothing that can
# change a computed value (no -ffast-math): save states and any future link mode need two
# machines to stay bit identical.
device_cflags="-flto=auto"

usage() {
	echo "usage: $0 stamp | build WORKDIR OUT" >&2
	exit 2
}

sha256() {
	if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1"; else shasum -a 256 "$1"; fi |
		cut -d' ' -f1
}

stamp() {
	echo "commit=$commit"
	echo "source=https://github.com/libretro/mgba/tree/$commit"
	echo "triple=$triple"
	echo "device_cflags=$device_cflags"
	for p in "$here"/*.patch; do
		echo "patch=$(basename "$p") sha256:$(sha256 "$p")"
	done
}

build() {
	work="$1" out="$2"
	src="$work/mgba"
	obj="$work/build"

	command -v cmake >/dev/null 2>&1 || { echo "cmake missing (run inside build/cross.Dockerfile)" >&2; exit 1; }

	# Pristine at the pin on every run, so a patch is never applied on top of itself and a
	# moved pin never builds over the previous checkout. The checkout's own .git is made first,
	# so a missing one cannot send git up into SLOT2's repository instead.
	mkdir -p "$src"
	[ -d "$src/.git" ] || git init -q "$src"
	git -C "$src" cat-file -e "$commit^{commit}" 2>/dev/null ||
		git -C "$src" fetch -q --depth 1 https://github.com/libretro/mgba "$commit"
	git -C "$src" checkout -q --force --detach "$commit"
	git -C "$src" clean -q -fdx
	for p in "$here"/*.patch; do
		git -C "$src" apply "$p"
	done

	rm -rf "$obj"
	set -- -S "$src" -B "$obj" -DLIBMGBA_ONLY=ON -DBUILD_LIBRETRO=ON -DCMAKE_BUILD_TYPE=Release \
		-DCMAKE_C_FLAGS="$device_cflags"
	if [ -n "$triple" ]; then
		set -- "$@" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=aarch64 \
			-DCMAKE_C_COMPILER="$triple-gcc" -DCMAKE_CXX_COMPILER="$triple-g++" \
			-DCMAKE_AR="$(command -v "$triple-gcc-ar" || command -v "$triple-ar")" \
			-DCMAKE_RANLIB="$(command -v "$triple-gcc-ranlib" || command -v "$triple-ranlib")" \
			-DCMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER -DCMAKE_FIND_ROOT_PATH_MODE_LIBRARY=ONLY \
			-DCMAKE_FIND_ROOT_PATH_MODE_INCLUDE=ONLY
	fi
	cmake "$@" >/dev/null
	cmake --build "$obj" --target mgba_libretro --parallel "$(getconf _NPROCESSORS_ONLN)" >/dev/null

	if [ -f "$obj/mgba_libretro.so" ]; then
		mkdir -p "$(dirname "$out")"
		cp "$obj/mgba_libretro.so" "$out"
		"${triple:+$triple-}strip" --strip-unneeded "$out" 2>/dev/null || true
		stamp >"$out.meta"
		file "$out"
		return
	fi
	echo "cmake finished without producing mgba_libretro.so" >&2
	exit 1
}

case "${1:-}" in
stamp)
	[ $# -eq 1 ] || usage
	stamp
	;;
build)
	[ $# -eq 3 ] && [ -n "$2" ] && [ -n "$3" ] || usage
	build "$2" "$3"
	;;
*)
	usage
	;;
esac
