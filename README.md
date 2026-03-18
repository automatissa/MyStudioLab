# MyStudioLab

Free, offline, open-source screen recorder with auto-zoom — for Windows, macOS, and Linux.

The open-source alternative to Screen Studio, FocuSee, and Rapidemo.
No paywalls. No watermarks. No accounts. No internet required.

---

## Features

- **Desktop GUI** — dark-themed native app (egui/eframe), no browser, no Electron
- **Auto-zoom** — smooth zoom-in on clicks, eased transitions, fully configurable
- **Hardware-accelerated encoding** — NVENC (NVIDIA), AMF (AMD), VideoToolbox (Apple), VAAPI (Linux), libx265 fallback
- **H.265/MP4 or AV1/WebM** output up to 4K @ 60 fps
- **100% offline** — zero telemetry, zero network calls, all processing local
- **Cross-platform** — Windows (WGC API), macOS (ScreenCaptureKit via scap), Linux (X11 + Wayland/PipeWire)
- **CLI** — full headless control for scripting and automation

---

## Quick start

### Prerequisites

| Tool | Notes |
|------|-------|
| [Rust](https://rustup.rs) 1.85+ | stable toolchain |
| [FFmpeg](https://ffmpeg.org/download.html) | binary on `PATH` — no dev package needed |
| LLVM/Clang | required by `windows-capture` bindgen on Windows |

On **Windows** the recommended FFmpeg build is the [gyan.dev full shared build](https://www.gyan.dev/ffmpeg/builds/).

### Build from source

```sh
git clone https://github.com/your-org/mystudiolab
cd mystudiolab
cargo build --release
```

| Binary | Path | Description |
|--------|------|-------------|
| GUI | `target/release/mystudiolab-gui.exe` | Desktop app (recommended) |
| CLI | `target/release/mystudiolab.exe` | Headless / scriptable |

---

## GUI

Launch the desktop app:

```sh
./target/release/mystudiolab-gui.exe
```

**⏺ Record tab** — set output path, display, FPS and zoom level, then click Start.
**⚙ Settings tab** — fine-tune zoom transitions, hold duration, and audio sources.

Click **Start Recording**, do your thing, click **Stop Recording** — the MP4 is saved automatically.

---

## CLI

```sh
# Record primary display at 60 fps with auto-zoom
mystudiolab record

# Custom output, zoom level, and FPS
mystudiolab record --output ~/Videos/demo.mp4 --zoom 2.5 --fps 30

# No zoom
mystudiolab record --no-zoom

# Capture a secondary display
mystudiolab record --display 1

# With microphone
mystudiolab record --mic

# AV1/WebM output
mystudiolab record --output recording.webm
```

Press **Ctrl-C** to stop. The file is finalized on exit.

```
Options:
  -o, --output <OUTPUT>    Output file [default: recording.mp4]
  -d, --display <DISPLAY>  Display index (0 = primary) [default: 0]
      --fps <FPS>          Frames per second [default: 60]
      --zoom <ZOOM>        Max zoom level (1.0 = off) [default: 2.0]
      --no-zoom            Disable auto-zoom
      --mic                Capture microphone audio
      --system-audio       Capture system audio (loopback)
```

---

## How it works

```
Screen (WGC / scap)
       │  RawFrame (BGRA)
       ▼
  ZoomProcessor  ◄── MouseTracker (rdev)
       │  ZoomedFrame (crop + upscale)
       ▼
    Encoder
       │  raw BGRA → stdin pipe
       ▼
   ffmpeg process  (hevc_nvenc / hevc_amf / libx265 / …)
       │
       ▼
  output.mp4 / output.webm
```

The encoder spawns `ffmpeg` as a child process and pipes raw BGRA frames into its stdin — no FFmpeg C-library linking required, full access to all hardware encoders in the installed binary.

---

## Project structure

```
crates/
  capture/   Screen capture — Windows WGC API, macOS/Linux scap
  zoom/      Auto-zoom state machine, easing functions, rdev mouse tracker
  encode/    ffmpeg subprocess encoder, hardware backend auto-detection
  audio/     Microphone + system loopback (cpal) — stub, v1.1
  cli/       mystudiolab binary — headless pipeline wiring
  gui/       mystudiolab-gui binary — egui/eframe desktop app
```

---

## Roadmap

- **v1.0** ✅ — GUI + CLI, screen capture, auto-zoom, H.265/AV1 hardware encoding
- **v1.1** — audio capture (mic + loopback) muxed into output file
- **v1.2** — display enumeration UI, window capture target
- **v1.3** — high-quality Lanczos upscaler
- **v2** — basic video editor (trim, cut) — fully offline
- **v3** — effects, transitions, color grading — fully offline
- **v4** — local plugin marketplace

---

## Dev environment (Windows)

```sh
# Environment variables (set in .cargo/config.toml — no manual export needed)
FFMPEG_DIR    = C:\ffmpeg
LIBCLANG_PATH = C:\Program Files\LLVM\bin

# Type-check everything
cargo check --workspace

# Build both binaries
cargo build --release -p mystudiolab
cargo build --release -p mystudiolab-gui

# Run the GUI
.\target\release\mystudiolab-gui.exe
```

---

## License

[MIT](LICENSE)
