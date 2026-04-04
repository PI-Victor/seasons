Seasons
-------

An unofficial cross-platform Tauri app for controlling Philips Hue lights, devices, scenes, automations, and Entertainment audio sync.

The project is primarily aimed at Linux, especially for features that are usually better covered by the macOS Hue desktop app.

## Features

- Room-first Hue control UI
- Scene browsing, activation, creation, and deletion
- Device controls grouped by room or by type
- Hue automations with detail inspection
- Entertainment audio sync over PipeWire on Linux

## Linux dependencies

You need both the normal Tauri/WebKitGTK build stack and the PipeWire development package for audio sync.

### Debian / Ubuntu

```bash
sudo apt update
sudo apt install \
  build-essential \
  curl \
  file \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  libpipewire-0.3-dev
```

### Fedora

```bash
sudo dnf install \
  cairo-gobject-devel \
  gcc-c++ \
  glib2-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  libxdo-devel \
  openssl-devel \
  pipewire-devel \
  pkgconf-pkg-config \
  webkit2gtk4.1-devel
```

### Arch Linux

```bash
sudo pacman -S --needed \
  base-devel \
  libappindicator-gtk3 \
  libpipewire \
  openssl \
  webkit2gtk-4.1 \
  xdotool
```

## Rust / frontend tools

```bash
rustup target add wasm32-unknown-unknown
cargo install tauri-cli --version "^2.0.0" --locked
cargo install trunk
cargo install wasm-bindgen-cli --version 0.2.117
```

## Development

Run the app with:

```bash
cargo tauri dev
```

On some Linux graphics stacks, WebKitGTK needs the DMA-BUF renderer disabled:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo tauri dev
```

## Build

```bash
cargo tauri build
```

## Notes

- Entertainment audio sync requires a Hue Entertainment area created in the official Hue app.
- Entertainment audio sync on Linux currently uses PipeWire output capture.
- The saved bridge session and app state are stored in XDG config/data locations, not in browser local storage.
