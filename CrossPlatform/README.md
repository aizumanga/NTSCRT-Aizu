# NTSCRT for Windows and Linux

This is the user and build guide for the Rust frontend included in the Windows and Linux packages. It preserves the original macOS app and implements this portable still-image pipeline:

1. `ntsc-rs` analog NTSC/VHS degradation.
2. Configurable downscaling to a retro resolution.
3. A RetroArch CRT preset rendered through `librashader` and WGPU.
4. Side-by-side preview and PNG export.

## Current feature support

| Feature | Windows/Linux |
| --- | --- |
| PNG, JPEG, WebP, BMP and TGA input | Yes |
| GIF input | First frame only |
| Complete `ntsc-rs` settings panel | Yes |
| Seven bundled CRT presets | Yes |
| Configurable downscale and output size | Yes |
| Side-by-side source/result preview | Yes |
| PNG export | Yes |
| Save/load combined JSON presets | Yes |
| Video, animated GIF and audio | Not yet |
| Timeline/keyframes | Not yet |
| Per-shader runtime parameter controls | Not yet |

The missing features currently depend on the macOS-only AVFoundation, CoreVideo, Metal, and SwiftUI implementation. The Windows/Linux app is fully usable for still images, but it is not yet a complete replacement for the macOS version.

## Installing on Windows

1. Open [Releases](../../../releases/latest), expand **Assets**, and download `NTSCRT-Windows-x64.zip`.
2. Right-click the downloaded ZIP and choose **Extract All**.
3. Open the extracted `NTSCRT-Windows-x64` folder.
4. Confirm that it contains both `ntscrt.exe` and `slang-shaders`.
5. Double-click `ntscrt.exe`.

Do not launch the app from inside the ZIP. Do not move `ntscrt.exe` away from `slang-shaders`; the bundled CRT presets will not load if that folder is missing.

The executable is not currently code-signed, so Windows SmartScreen may warn that the publisher is unknown. If the file came directly from this repository, click **More info → Run anyway**. Do not bypass the warning for downloads from third-party mirrors.

No separate Rust, Python, RetroArch, or `ntsc-rs` installation is required for the packaged version.

## Installing on Linux

Open [Releases](../../../releases/latest) and download `NTSCRT-Linux-x86_64.tar.gz`, then run:

```sh
tar -xzf NTSCRT-Linux-x86_64.tar.gz
cd NTSCRT-Linux-x86_64
./ntscrt
```

Keep `ntscrt` beside the bundled `slang-shaders` folder. If the executable bit was lost while copying or extracting the file:

```sh
chmod +x ntscrt
./ntscrt
```

The app supports X11 and Wayland. It needs a working graphics driver; Vulkan is recommended. GNOME and KDE normally already provide the desktop portal used by the Open/Save dialogs. Minimal window-manager setups may need `xdg-desktop-portal` and an appropriate backend.

Common portal packages:

- Arch Linux: `xdg-desktop-portal` plus `xdg-desktop-portal-gtk`, `xdg-desktop-portal-gnome`, or `xdg-desktop-portal-kde`.
- Debian/Ubuntu: `xdg-desktop-portal` plus a matching `xdg-desktop-portal-*` backend.
- Fedora: `xdg-desktop-portal` plus a matching backend.

No separate Rust, Python, RetroArch, or `ntsc-rs` installation is required for the packaged version.

## Processing an image

1. Click **Open image** and select a PNG, JPEG, WebP, BMP, TGA, or GIF.
2. Choose a **Retro width**. This is the low internal horizontal resolution seen by the CRT shader; `320` is a useful general starting point.
3. Choose an **Output width**. The output height is calculated automatically to preserve the source aspect ratio.
4. Choose a **Resize** filter:
   - **Nearest** keeps hard pixel edges and is best for pixel art.
   - **Area** is a neutral starting point for screenshots, drawings, and photos.
   - **Bilinear**, **Bicubic**, and **Lanczos** provide progressively smoother alternatives.
5. Choose one of the seven **CRT shader** presets.
6. Leave **Enable ntsc-rs** checked for VHS/composite artifacts, or disable it to apply only downscaling and the CRT shader.
7. Adjust the settings under **VHS / NTSC**. Hover over a control to see its description when available.
8. Click **Render preview**. An asterisk in `Render preview *` means the controls changed and the preview is out of date.
9. Compare the original under **Source** with the result under **Processed**.
10. Click **Export PNG** and choose where to save the rendered image.

The status bar at the bottom reports loading, rendering, export, missing-shader, and graphics errors. Rendering is not automatic after every slider movement so that expensive settings can be adjusted together.

## Suggested starting settings

- **Pixel art:** Retro width near the artwork's native width, **Nearest**, and an output width at least three times the retro width.
- **Anime screenshots or illustrations:** Retro width `320`–`640`, **Area** or **Bicubic**, output width `1280` or higher.
- **Photos:** Start with **Area**, a higher retro width, and reduce the stronger VHS noise controls.
- **CRT without VHS damage:** Clear **Enable ntsc-rs** and keep a CRT shader selected.

Very large source images can take longer because `ntsc-rs` processes the source at full resolution before it is downscaled.

## Presets

**Save preset** writes a JSON file containing:

- whether `ntsc-rs` is enabled;
- all NTSC/VHS values;
- retro width and resize filter;
- output width;
- the selected CRT shader.

**Load preset** restores those values. A preset does not contain the source image or rendered output, so it is safe to reuse with other images. After loading a preset, open an image if necessary and click **Render preview**.

## Troubleshooting

### “CRT shaders were not found”

The complete extracted package must look like this:

```text
NTSCRT-Windows-x64/       or NTSCRT-Linux-x86_64/
├── ntscrt.exe            or ntscrt
└── slang-shaders/
    ├── crt/
    └── include/
```

Extract the archive again if `slang-shaders` is missing. Advanced users can point to another compatible shader tree with `NTSCRT_SHADER_DIR`.

PowerShell:

```powershell
$env:NTSCRT_SHADER_DIR = "C:\absolute\path\to\slang-shaders"
.\ntscrt.exe
```

Linux:

```sh
NTSCRT_SHADER_DIR=/absolute/path/to/slang-shaders ./ntscrt
```

### The app opens, but rendering reports no compatible graphics adapter

Update the GPU driver and ensure Vulkan is installed and working. Virtual machines, remote desktop sessions, old integrated GPUs, and incomplete drivers may expose WGPU without all features required by the CRT shaders.

### Open or Save does nothing on Linux

Install `xdg-desktop-portal` and a portal backend matching your desktop, then log out and back in. This is especially common with minimal window managers.

### Windows blocks the first launch

Verify that the ZIP came from this repository's Releases page. The current executable is unsigned; choose **More info → Run anyway** in SmartScreen only after verifying the source.

### The preview did not change

After changing a setting, click **Render preview** again. The `*` on the button indicates that the displayed output uses older settings.

## Building from source

The packaged app is recommended for normal use. Source builds are intended for contributors and do require Rust plus platform development libraries.

First clone the repository with its submodules:

```sh
git clone --recurse-submodules https://github.com/aizumanga/NTSCRT-Aizu.git
cd NTSCRT-Aizu
```

If the repository was cloned without `--recurse-submodules`:

```sh
git submodule update --init --recursive
```

Install the current stable Rust toolchain from [rustup.rs](https://rustup.rs/).

### Windows source build

Install the Visual Studio Build Tools with the **Desktop development with C++** workload, open PowerShell in the repository root, and run:

```powershell
cargo build --release --locked --manifest-path CrossPlatform\Cargo.toml
.\CrossPlatform\target\release\ntscrt.exe
```

Run from the repository root so the app can find `Vendor\slang-shaders`. The executable is created at `CrossPlatform\target\release\ntscrt.exe`.

### Linux source build

Install the compiler, `pkg-config`, and X11/Wayland development packages used by the windowing backend. For Debian/Ubuntu:

```sh
sudo apt update
sudo apt install build-essential pkg-config libwayland-dev libx11-dev \
  libxcursor-dev libxi-dev libxkbcommon-dev libxrandr-dev
```

Then build and run from the repository root:

```sh
cargo build --release --locked --manifest-path CrossPlatform/Cargo.toml
./CrossPlatform/target/release/ntscrt
```

The executable is created at `CrossPlatform/target/release/ntscrt`.

## Developer checks

The same main checks used by CI are:

```sh
cargo fmt --manifest-path CrossPlatform/Cargo.toml -- --check
cargo check --locked --manifest-path CrossPlatform/Cargo.toml
cargo test --locked --manifest-path CrossPlatform/Cargo.toml
cargo clippy --locked --manifest-path CrossPlatform/Cargo.toml -- -D warnings
```

The GitHub Actions workflow runs the check and test suite on both Ubuntu and Windows, produces `.tar.gz` and `.zip` packages, and attaches both packages to releases created from `v*` tags.
