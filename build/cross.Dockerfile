# Cross-compile box for the H700: x86_64 host, aarch64 target, glibc 2.31 floor.
#
# Bullseye on purpose. BaseOS's userland is harvested from Anbernic's stock firmware (glibc
# ~2.35); a binary built against 2.31 runs there, one built against anything newer might not.
# The same image serves `cargo build --target aarch64-unknown-linux-gnu` (pure Rust, every
# system library is dlopen'd at runtime) and the C cores (cmake/make with the cross gcc).
#
#   docker build -t slot2-cross -f build/cross.Dockerfile build
#   docker run --rm -v "${PWD}:/src" -w /src slot2-cross \
#       cargo build --profile device --target aarch64-unknown-linux-gnu \
#       --target-dir target-device -p slot2 --no-default-features --features device

FROM rust:1-bullseye

# Bullseye is at the end of LTS and its security mirror sometimes serves an index whose
# packages are already gone (404 on fetch). A build box does not need security updates, so
# on failure drop that source and try again from main alone.
RUN set -eux; \
    pkgs="gcc-aarch64-linux-gnu g++-aarch64-linux-gnu cmake make git file pkg-config ca-certificates"; \
    install() { apt-get update && apt-get install -y --no-install-recommends -o Acquire::Retries=3 $pkgs; }; \
    install || { sed -i '/security/d' /etc/apt/sources.list; rm -rf /var/lib/apt/lists/*; install; }; \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add aarch64-unknown-linux-gnu \
    && rustup component add rustfmt clippy

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
    AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar \
    CARGO_HOME=/cargo

# Keep the registry and git caches in a named volume so repeated builds do not refetch.
VOLUME ["/cargo"]
