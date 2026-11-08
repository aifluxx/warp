# syntax=docker/dockerfile:1

# Company factory page. This image does not contain warp-tui-oss or ttyd.

FROM rust:1.92.0-bookworm AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        cmake \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY app ./app
RUN cargo build -p company_factory --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 factory

COPY --from=build /src/target/release/company_factory /usr/local/bin/company_factory

USER factory
EXPOSE 8792
ENTRYPOINT ["/usr/local/bin/company_factory"]
