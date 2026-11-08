# syntax=docker/dockerfile:1

# Headless Warp TUI for a small Linux VPS.
#
# This image builds `warp-tui-oss` (the open-source channel of the headless
# terminal) and serves it on a private web terminal. It does not include
# Warp's cloud backend. See the "Self-hosting the headless TUI" section in
# the repository README.
#
# The builder stage needs several gigabytes of RAM and tens of gigabytes of
# disk. Do not run it on srv1307450 while the aifluxx stacks are up.

FROM ubuntu:24.04@sha256:fe81560990fae86dc4743202b2cb4d3df98d0bcf949173cecef15ce1127ef852 AS build

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH} \
    CARGO_TERM_COLOR=never \
    CARGO_INCREMENTAL=0

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        cmake \
        curl \
        git \
        libclang-dev \
        libexpat1-dev \
        libfontconfig1-dev \
        libfreetype-dev \
        libgit2-dev \
        libssl-dev \
        pkg-config \
        python-is-python3 \
        unzip \
        zlib1g-dev

# Ubuntu 24.04's protoc is new enough for this tree. Install the same 25.1
# release script/linux/install_build_deps uses so proto3 optional fields match
# local and CI builds.
ARG PROTOC_VERSION=25.1
RUN set -eu; \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
      amd64) protoc_zip="protoc-${PROTOC_VERSION}-linux-x86_64.zip" ;; \
      arm64) protoc_zip="protoc-${PROTOC_VERSION}-linux-aarch_64.zip" ;; \
      *) echo "unsupported architecture: $arch" >&2; exit 1 ;; \
    esac; \
    curl --proto '=https' --tlsv1.2 -fsSL \
      -o /tmp/protoc.zip \
      "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/${protoc_zip}"; \
    unzip -o /tmp/protoc.zip -d /usr/local bin/protoc 'include/*'; \
    rm /tmp/protoc.zip; \
    protoc --version

RUN curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain 1.92.0 --profile minimal --no-modify-path \
    && rustup default 1.92.0 \
    && rustc --version \
    && cargo --version

WORKDIR /src
COPY . .

# rust-toolchain.toml also requests rustfmt, clippy, and rust-analyzer. Those
# are not needed to produce the TUI binary, and rust-analyzer is large.
RUN rm -f rust-toolchain.toml

ARG WARP_CARGO_PROFILE=release-tui
ARG CARGO_BUILD_JOBS=2
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/src/target,sharing=locked \
    set -eu; \
    case "$WARP_CARGO_PROFILE" in \
      dev) profile_dir=debug ;; \
      *) profile_dir="$WARP_CARGO_PROFILE" ;; \
    esac; \
    cargo build --locked \
      --profile "$WARP_CARGO_PROFILE" \
      -p warp_tui \
      --bin warp-tui-oss \
      --features standalone; \
    mkdir -p /opt/warp-oss; \
    cp "/src/target/${profile_dir}/warp-tui-oss" /opt/warp-oss/warp-tui-oss; \
    chmod 755 /opt/warp-oss/warp-tui-oss; \
    export HOME=/tmp; \
    export XDG_CONFIG_HOME=/tmp/warp-xdg/config; \
    export XDG_DATA_HOME=/tmp/warp-xdg/data; \
    export XDG_STATE_HOME=/tmp/warp-xdg/state; \
    export XDG_CACHE_HOME=/tmp/warp-xdg/cache; \
    mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_STATE_HOME" "$XDG_CACHE_HOME"; \
    NO_LICENSES=1 \
      SETTINGS_SCHEMA_EXECUTABLE=/opt/warp-oss/warp-tui-oss \
      /src/script/prepare_bundled_resources /opt/warp-oss/resources oss; \
    chmod -R a+rX /opt/warp-oss

FROM ubuntu:24.04@sha256:fe81560990fae86dc4743202b2cb4d3df98d0bcf949173cecef15ce1127ef852 AS runtime

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    TERM=xterm-256color \
    TTYD_PORT=7681

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        libexpat1 \
        libfontconfig1 \
        libfreetype6 \
        libssl3 \
        libwayland-client0 \
        libx11-6 \
        libxcb1 \
        libxcursor1 \
        libxi6 \
        libxkbcommon0 \
        libxkbcommon-x11-0 \
        ttyd \
        zlib1g \
    && ttyd --version \
    && if ! id warp >/dev/null 2>&1; then \
         useradd --uid 65532 --user-group --create-home --home-dir /var/lib/warp-tui --shell /usr/sbin/nologin warp; \
       fi \
    && mkdir -p /var/lib/warp-tui \
    && chown warp:warp /var/lib/warp-tui

COPY --link --from=build /opt/warp-oss /opt/warp-oss
COPY --link --chmod=755 deploy/vps/entrypoint.sh /usr/local/bin/warp-oss-entrypoint

RUN ldd /opt/warp-oss/warp-tui-oss \
    && ! ldd /opt/warp-oss/warp-tui-oss | grep -q 'not found'

EXPOSE 7681
VOLUME ["/var/lib/warp-tui"]
ENTRYPOINT ["/usr/local/bin/warp-oss-entrypoint"]
CMD ["serve"]
