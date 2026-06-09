# Build stage
FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM rust:1.93.0-bookworm AS builder
WORKDIR /usr/src/litehook

# Copy xx scripts
COPY --from=xx / /

ARG TARGETPLATFORM
ARG BUILDPLATFORM

# Install deps
RUN apt-get update && apt-get install -y \
    ca-certificates \
    llvm \
    clang \
    lld \
    gcc-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*
RUN xx-apt-get install -y libc6-dev gcc-12 libgcc-12-dev

RUN rustup target add $(xx-cargo --print-target-triple)

COPY . .

# Build
RUN xx-cargo build --release --target-dir ./build && \
    xx-verify ./build/$(xx-cargo --print-target-triple)/release/litehook && \
    llvm-strip ./build/$(xx-cargo --print-target-triple)/release/litehook && \
    cp ./build/$(xx-cargo --print-target-triple)/release/litehook /litehook-out && \
    # Copy tdjson shared library to /usr/local/lib
    find ./build/$(xx-cargo --print-target-triple)/release/build -name "libtdjson.so.*" -exec cp {} /usr/local/lib/ \;

# Export binary
FROM scratch AS export
COPY --from=builder /litehook-out /litehook

# Runtime stage
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libstdc++6 \
    libc++1 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /litehook-out /litehook
COPY --from=builder /usr/local/lib/libtdjson.so.* /usr/local/lib/
RUN ldconfig
ENTRYPOINT ["/litehook"]
