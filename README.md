# Kagi Assistant Desktop

A desktop application for [Kagi Assistant](https://kagi.com/assistant) built with [Tauri](https://tauri.app/).

* Note: After first time logging in, you be navigated to the kagi.com homepage, after you restart the application you will be sent directly to /assistant each time

![image](https://github.com/user-attachments/assets/4946da7f-94a8-41ff-b3ce-9ab0e8f07d4c)

## Install

### Binaries (Mac, Linux, Windows)

Download from https://github.com/0xGingi/kagi-assistant-desktop/releases/latest

### Arch Linux (AUR)

AUR Package: https://aur.archlinux.org/packages/kagi-assistant-desktop-git

```bash
paru -S kagi-assistant-desktop-git
```

## Build

### Prerequisites

- [Rust](https://www.rust-lang.org/)
- [Bun](https://bun.sh/)

### Building

```bash
bun install

bun run tauri build --bundles
```

This will build to `src-tauri/target/release/bundle`.
