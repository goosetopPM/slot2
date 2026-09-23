#!/bin/sh
# Shared build steps for the libretro cores that ship a plain Makefile. mGBA is the odd one
# out — it needs cmake and has its own script; everything else is the same three moves
# (checkout at a pin, apply the patches beside the script, make) with a different repo,
# makefile and output name.
#
# A core's build.sh sets the variables below and then sources this file; it must not define
# `stamp` or `build` itself.
#
#   CORE_REPO       git URL
#   CORE_TARGET     the .so the makefile produces, relative to CORE_MAKE_DIR
#   CORE_MAKE_DIR   directory to run make in, relative to the checkout (default ".")
#   CORE_MAKEFILE   makefile name (default "Makefile.libretro")
#   CORE_MAKE_ARGS  anything else to put on the make command line (optional)
#
# The interface is the same as cores/mgba/build.sh:
#
#   build.sh stamp                      print what a build would record (.meta)
#   build.sh build WORKDIR OUT          checkout, patch, build into WORKDIR, write OUT and OUT.meta
#
# Environment:
#   CROSS_TRIPLE   toolchain prefix, default aarch64-linux-gnu. Set empty to build natively.
set -eu

commit="$(tr -d ' \n\r' <"$here/commit")"
triple="${CROSS_TRIPLE-aarch64-linux-gnu}"
: "${CORE_MAKE_DIR:=.}"
: "${CORE_MAKEFILE:=Makefile.libretro}"
: "${CORE_MAKE_ARGS:=}"

# These makefiles pick their own optimisation level, and they build it out of `CFLAGS +=`.
# Setting CFLAGS on the make command line replaces that instead of adding to it, taking the
# core's own -D defines with it — fceumm loses FCEU_VERSION_NUMERIC and stops compiling.
# So the flags stay as the core's author wrote them. mGBA carries an -flto=auto that was
# measured to be worth 3-5% a frame; if one of these turns out too slow on the H700 it can
# earn the same treatment through the environment, with a number to justify it.
device_cflags="core defaults"

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
	echo "source=$CORE_REPO/tree/$commit"
	echo "triple=$triple"
	echo "device_cflags=$device_cflags"
	for p in "$here"/*.patch; do
		[ -e "$p" ] || continue
		echo "patch=$(basename "$p") sha256:$(sha256 "$p")"
	done
}

build() {
	work="$1" out="$2"
	src="$work/src"

	# Pristine at the pin on every run, so a patch is never applied on top of itself and a
	# moved pin never builds over the previous checkout. The checkout's own .git is made
	# first, so a missing one cannot send git up into SLOT2's repository instead.
	mkdir -p "$src"
	[ -d "$src/.git" ] || git init -q "$src"
	git -C "$src" cat-file -e "$commit^{commit}" 2>/dev/null ||
		git -C "$src" fetch -q --depth 1 "$CORE_REPO" "$commit"
	git -C "$src" checkout -q --force --detach "$commit"
	git -C "$src" clean -q -fdx
	for p in "$here"/*.patch; do
		[ -e "$p" ] || continue
		git -C "$src" apply "$p"
	done

	# platform=unix is what every one of these makefiles calls "a Linux .so"; the toolchain
	# comes from CC/CXX/AR so the same recipe cross-compiles. The makefiles that sniff
	# `uname -m` for tuning are steered with ARCH, which they all read before falling back.
	set -- -C "$src/$CORE_MAKE_DIR" -f "$CORE_MAKEFILE" platform=unix
	if [ -n "$triple" ]; then
		set -- "$@" CC="$triple-gcc" CXX="$triple-g++" \
			AR="$(command -v "$triple-gcc-ar" || command -v "$triple-ar")" \
			RANLIB="$(command -v "$triple-gcc-ranlib" || command -v "$triple-ranlib")" \
			ARCH=arm64
	fi
	# shellcheck disable=SC2086 # CORE_MAKE_ARGS is a word list on purpose
	make "$@" $CORE_MAKE_ARGS -j"$(getconf _NPROCESSORS_ONLN)" >/dev/null

	produced="$src/$CORE_MAKE_DIR/$CORE_TARGET"
	if [ -f "$produced" ]; then
		mkdir -p "$(dirname "$out")"
		cp "$produced" "$out"
		"${triple:+$triple-}strip" --strip-unneeded "$out" 2>/dev/null || true
		stamp >"$out.meta"
		file "$out"
		return
	fi
	echo "make finished without producing $CORE_TARGET" >&2
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
