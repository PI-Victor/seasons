FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:/usr/local/rustup/bin:${PATH} \
    RUSTC_WRAPPER=

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    file \
    git \
    libayatana-appindicator3-dev \
    libgtk-3-dev \
    libjavascriptcoregtk-4.1-dev \
    libpipewire-0.3-dev \
    librsvg2-dev \
    libsoup-3.0-dev \
    libssl-dev \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    patchelf \
    pkg-config \
    rpm \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain stable \
    && rustup target add wasm32-unknown-unknown \
    && cargo install --locked trunk \
    && cargo install --locked tauri-cli --version "^2.0.0"

WORKDIR /work
COPY . .

# Produces Linux artifacts under:
# /work/src-tauri/target/release/bundle
RUN cargo tauri build

CMD ["/bin/bash"]
