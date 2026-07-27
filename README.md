# NTSCRT

![NTSCRT — full VHS + CRT pipeline on the left of the split, untouched source on the right](docs/header.webp)

**Make any image or video look like it's playing on a 1980s TV.** NTSCRT runs your media through a real analog signal emulation ([ntsc-rs](https://github.com/ntsc-rs/ntsc-rs) — composite artifacts, tape noise, head switching) and then through RetroArch's CRT shaders (via [librashader](https://github.com/SnowflakePowered/librashader) — scanlines, phosphor masks, glow), frame-identical to RetroArch itself.

Full disclosure: **this is two much better projects hacked together.** All of the actual image magic belongs to ntsc-rs and the RetroArch shader community; NTSCRT connects them into one pipeline:

```
your image/video → NTSC/VHS signal degradation (full res) → downscale to retro resolution → CRT shader → screen
```

## Download

Open [**Releases**](../../releases/latest), expand **Assets**, and download the package for your operating system.

| Platform | Download | Requirements |
| --- | --- | --- |
| Windows | `NTSCRT-Windows-x64.zip` | Windows 10/11 64-bit and a current graphics driver |
| Linux | `NTSCRT-Linux-x86_64.tar.gz` | 64-bit Linux, X11 or Wayland, and a current Vulkan-capable graphics driver |
| macOS | Universal DMG | macOS 14+, Apple Silicon or Intel |

If a Windows/Linux release has not been published yet, see [building from source](CrossPlatform/README.md#building-from-source). The automated packages are produced by the [Cross-platform workflow](../../actions/workflows/cross-platform.yml) whenever a release tag is created.

The Windows/Linux frontend currently supports the complete still-image pipeline and PNG export. Video, animated GIF export, timeline/keyframes, and individual CRT shader parameters remain macOS-only while their AVFoundation/Metal implementation is ported. See the [full feature table](CrossPlatform/README.md#current-feature-support).

> **Intel Mac note:** I build and test NTSCRT on Apple Silicon and haven't personally tested the Intel build. Intel support exists thanks to a contributed fix ([#1](../../pull/1)) verified by its author on an Intel iMac Pro — if something misbehaves on your Intel Mac, please open an issue.

## Windows quick start

1. Download `NTSCRT-Windows-x64.zip` from **Releases**.
2. Right-click the ZIP, choose **Extract All**, and open the extracted folder.
3. Keep `ntscrt.exe` and the `slang-shaders` folder together. Do not run the app from inside the ZIP or move only the EXE.
4. Double-click `ntscrt.exe`.
5. Click **Open image**, adjust the settings, click **Render preview**, and then **Export PNG**.

The release is not code-signed. Windows SmartScreen may show an “unrecognized app” warning on the first launch. If you downloaded it from this repository, choose **More info → Run anyway**. You should never bypass that warning for a copy obtained elsewhere.

## Linux quick start

Download the archive, then extract and run it:

```sh
tar -xzf NTSCRT-Linux-x86_64.tar.gz
cd NTSCRT-Linux-x86_64
./ntscrt
```

Keep the `ntscrt` executable and the `slang-shaders` folder together. If your file manager removed the executable bit, run `chmod +x ntscrt`. On minimal desktops or window managers, the Open/Save dialogs may require `xdg-desktop-portal` plus a portal backend for your desktop.

See the [Windows/Linux user guide](CrossPlatform/README.md) for control explanations, recommended settings, troubleshooting, presets, and source-build instructions.

## Using the Windows/Linux app

1. **Open image** — choose a PNG, JPEG, WebP, BMP, TGA, or GIF. GIF input currently uses only its first frame.
2. Set **Retro width** — the low internal resolution fed to the CRT shader. `320` is a useful starting point.
3. Set **Output width** — the width of the exported PNG. Height is calculated automatically from the source aspect ratio.
4. Choose a resize filter and a **Built-in CRT preset**, then adjust the **VHS / NTSC** controls.
5. Click **Render preview** whenever an asterisk appears on that button.
6. Compare **Source** and **Processed**, then click **Export PNG**.

Choose the bundled CRT effect from **Built-in CRT preset** in the left sidebar.
**Save preset file…** stores the complete Windows/Linux setup as JSON;
**Load preset file…** restores it later. Preset files do not embed the source image.

## Using the macOS app

**Toolbar** — file actions live in the window toolbar: **Open** (⌘O) an image (PNG/JPEG/HEIC) or video (MP4/MOV), save/load a **Preset** (your entire configuration — downscale + VHS + shader + view — as a JSON file), and **Export** (⌘E): stills to PNG; videos to H.264/HEVC .mp4, ProRes .mov (with audio), or animated **GIF**, at your choice of resolution and quality. Scanline detail is brutal on lossy codecs — use the High/Very high quality tiers, or ProRes when it's headed into an edit. Exports are deterministic: same settings + same frame = same pixels.

**Scanline banding.** CRT shaders draw scanlines in *output* pixels, so if the export height isn't a whole multiple of the downscale height, one source line covers a fractional number of rows and the scanlines group into visible bands. Exports handle this automatically — they render at a whole multiple and average down — so you can ask for any size. **Snap size to scanline grid** takes the other route: it rounds the output to the nearest size where every source line gets the same whole number of rows, which keeps scanlines at their crispest but changes your dimensions. As a rule of thumb, crisp scanlines want 3+ output rows per downscale line, so a 320px-wide downscale wants ~960px+ of output.

**GIF** gets its own width and frame rate (6/12/24/30 fps), because it doesn't behave like the video codecs: 256 colours and run-length compression versus full-frame analog noise means files run large — roughly 0.65–0.95 bytes per pixel per frame. A 5-second 480px GIF at 12 fps lands near 8 MB; at 1080px it would be 30 MB+, past what most platforms accept. The panel estimates the size before you export and warns past ~10 MB. GIF stores frame delays in hundredths of a second, so the rates land on that grid (12 fps plays at 12.5, 24 at 25, 30 at 33.3), and 60 fps isn't offered — GIF can't reliably go past 50.

**Preview** — the floating palette holds the display controls. **Compare** (split-square) divides the preview: full pipeline on the left of the line, untouched original on the right — drag the line to move the split. **Integer scale** (grid) locks the image to whole-pixel multiples for perfectly uniform scanlines. **Animate** (sparkles) runs the preview continuously so tape noise, jitter, and interlacing actually move — leave it on for the real experience. Zoom with the slider (or ⌥-scroll), hold Space to pan when zoomed. The palette fades out when the mouse goes idle; move the mouse to bring it back. Videos get a transport bar docked under the preview — play/pause plus a full-width, frame-accurate scrubber, with all effects applied during playback.

**Timeline** — with an image loaded, toggle **Timeline** in the toolbar to keyframe-animate the entire effect chain and render it as video: scrub the playhead, dial in a look, press **Keyframe** to set one, move the playhead, dial in another look, keyframe again. Everything keys together as one master keyframe — parameters you don't change between keys hold still automatically. Click a keyframe to jump to it, and any parameter you change from there updates that keyframe in place, the way After Effects and Premiere behave. Drag a diamond to retime it, pick its interpolation from the dropdown underneath (linear, ease in, ease out, ease in-out, hold), and set the video length and frame rate in the timeline itself — export uses those. Keyframe times are proportional, so changing the duration stretches the whole animation. Image sources can export video even without keyframes — tape noise, jitter, and interlacing animate on their own ("VHS motion").

**Sidebar** — the creative pipeline, top to bottom in signal order:

- **Source** — the loaded file (drag & drop onto the panel works too).
- **Downscale** — the retro horizontal resolution the CRT shader sees (SNES 256px, VGA 320px, or any custom width — height always follows your source's aspect ratio) and the resampling method. Nearest keeps pixels crunchy (best for pixel art), Nearest+ keeps the punch without shimmering on video, Area is the smooth neutral choice.
- **VHS (ntsc-rs)** — the analog signal stage: composite noise, chroma bleed, head switching, tracking noise, tape speed, edge wave, and about sixty more. These are ntsc-rs's own settings — preset JSON copy/pastes both ways with the [ntsc-rs desktop app](https://github.com/ntsc-rs/ntsc-rs/releases).
- **Shader** — seven RetroArch CRT presets (crt-royale, crt-hyllian, crt-aperture, crt-easymode, two crtglow variants, crtsim) with every runtime parameter exposed. Grayed-out controls tell you which switch activates them — many CRT parameters only apply when their feature (curvature, mask, geometry mode…) is on.

**Tips**

- Every value next to a slider is a text field — click and type exact numbers.
- The effect reads best on game-art-style content: dark scenes, bright sprites, hard edges. Photos work too, but analog artifacts live on contrast.
- High-resolution sources: turn on **Scale → Scale with video size** in the VHS panel so artifact sizes track your input, and expect the NTSC stage to take longer per frame.

## Limitations

- The Intel half of the universal Mac build is community-tested, not author-tested.
- Windows/Linux currently process still images only; video, timeline/keyframes, animated export, and individual CRT shader controls remain macOS-only.
- GIF files open on Windows/Linux, but only the first frame is processed.
- The NTSC stage runs on the CPU at the source's full resolution, so very large images may render slowly.
- A few crt-royale parameters are compile-time disabled in the macOS shader build.
- There is no undo; save presets before large experiments.

## Building from source

See [DEVELOPMENT.md](DEVELOPMENT.md) for the full macOS developer setup. Windows/Linux contributors can use the dedicated [cross-platform build guide](CrossPlatform/README.md#building-from-source).

## Credits

- [ntsc-rs](https://github.com/ntsc-rs/ntsc-rs) — the NTSC/VHS signal emulation (MIT/ISC/Apache-2.0)
- [librashader](https://github.com/SnowflakePowered/librashader) by SnowflakePowered — the RetroArch-compatible shader runtime (MPL-2.0)
- [libretro/slang-shaders](https://github.com/libretro/slang-shaders) and the RetroArch community — the CRT shaders themselves: crt-royale by TroggleMonkey, crt-easymode and crt-aperture by EasyMode, crt-hyllian by Hyllian, crtsim, crtglow (various licenses, largely GPL)
