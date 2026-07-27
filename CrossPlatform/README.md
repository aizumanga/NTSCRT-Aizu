# NTSCRT for Windows and Linux

This directory contains the Rust frontend used on Windows and Linux. It keeps
the original macOS SwiftUI application untouched and implements the portable
image pipeline with:

1. `ntsc-rs` for analog NTSC/VHS degradation.
2. A configurable retro-resolution downscale.
3. `librashader`'s WGPU runtime for the same RetroArch CRT presets.
4. PNG export and JSON presets.

## Current feature support

| Feature | Windows/Linux |
| --- | --- |
| PNG, JPEG, WebP, BMP, TGA, GIF first-frame input | Yes |
| Complete `ntsc-rs` settings panel | Yes |
| Seven bundled CRT presets | Yes |
| Configurable downscale and output size | Yes |
| Side-by-side source/result preview | Yes |
| PNG export | Yes |
| Save/load combined presets | Yes |
| Video/GIF animation and audio | Not yet |
| Timeline/keyframes | Not yet |
| Per-shader runtime parameter controls | Not yet |

The remaining features currently depend on the macOS-only AVFoundation,
CoreVideo, Metal, and SwiftUI implementation. They are intentionally not
advertised as supported by this frontend.

## Build

Install a current stable Rust toolchain, clone submodules, and build:

```sh
git submodule update --init --recursive
cargo build --release --locked --manifest-path CrossPlatform/Cargo.toml
```

The executable is written to:

- Linux: `CrossPlatform/target/release/ntscrt`
- Windows: `CrossPlatform\target\release\ntscrt.exe`

For a development checkout, start it from the repository root so it can find
`Vendor/slang-shaders`. A packaged build places `slang-shaders` beside the
executable. You can always override shader discovery:

```sh
NTSCRT_SHADER_DIR=/absolute/path/to/slang-shaders \
  CrossPlatform/target/release/ntscrt
```

PowerShell:

```powershell
$env:NTSCRT_SHADER_DIR = "C:\absolute\path\to\slang-shaders"
.\CrossPlatform\target\release\ntscrt.exe
```

## Linux runtime notes

The binary supports X11 and Wayland. A working Vulkan driver is recommended
for the WGPU CRT backend. The graphical file picker uses the desktop portal,
which is normally already installed by KDE Plasma and GNOME. Minimal window
manager setups may need an `xdg-desktop-portal` implementation.
