# MyStudioLab

Free, offline, open-source screen recorder with auto-zoom — for Windows, macOS, and Linux.

The open-source alternative to Screen Studio, FocuSee, and Rapidemo.
No paywalls. No watermarks. No accounts. No internet required.

---

## Features

- **Auto-zoom** — smooth zoom-in on clicks, eased transitions, configurable level and speed
- **Hardware-accelerated encoding** — NVENC (NVIDIA), AMF (AMD), VideoToolbox (Apple), VAAPI (Linux), libx265 fallback
- **H.265/MP4 or AV1/WebM** output up to 4K @ 60 fps
- **100% offline** — zero telemetry, zero network calls, all processing local
- **Cross-platform** — Windows (WGC API), macOS (ScreenCaptureKit via scap), Linux (X11 + Wayland/PipeWire)

---

## Install

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

The binary is at `target/release/mystudiolab` (or `mystudiolab.exe` on Windows).

---

## Usage

```sh
# Record primary display at 60 fps with auto-zoom (default)
mystudiolab record

# Custom output path and zoom level
mystudiolab record --output ~/Videos/demo.mp4 --zoom 2.5

# Record at 30 fps, no zoom
mystudiolab record --fps 30 --no-zoom

# Capture a specific display (0 = primary)
mystudiolab record --display 1 --output second-monitor.mp4

# With microphone audio
mystudiolab record --mic

# AV1/WebM output
mystudiolab record --output recording.webm
```

Press **Ctrl-C** to stop recording. The output file is written and finalized on exit.

### All options

```
Usage: mystudiolab record [OPTIONS]

Options:
  -o, --output <OUTPUT>    Output file [default: recording.mp4]
  -d, --display <DISPLAY>  Display index to capture (0 = primary) [default: 0]
      --fps <FPS>          Frames per second [default: 60]
      --zoom <ZOOM>        Maximum zoom level (1.0 = no zoom) [default: 2.0]
      --no-zoom            Disable auto-zoom entirely
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
       │  raw BGRA bytes → stdin
       ▼
   ffmpeg process  (hevc_nvenc / hevc_amf / libx265 / …)
       │
       ▼
  output.mp4 / output.webm
```

The encoder spawns `ffmpeg` as a child process and pipes raw BGRA frames into its stdin. This avoids FFmpeg C-library ABI issues on Windows while giving full access to every hardware encoder that the installed FFmpeg binary was compiled with.

---

## Project structure

```
crates/
  capture/   Screen capture (Windows WGC, macOS/Linux scap)
  zoom/      Auto-zoom state machine, easing, rdev mouse tracker
  encode/    ffmpeg subprocess encoder, hardware backend detection
  audio/     Microphone + system loopback capture (cpal) — in progress
  cli/       mystudiolab binary, end-to-end pipeline wiring
```

---

## Roadmap

- **v0.1** — screen capture + auto-zoom + H.265 encoding (current)
- **v0.2** — audio capture (mic + loopback), muxed into output file
- **v0.3** — display enumeration, window capture target
- **v1.0** — high-quality Lanczos upscaler, configurable zoom easing
- **v2** — basic video editor (trim, cut) — fully offline
- **v3** — effects, transitions, color grading — fully offline
- **v4** — local plugin marketplace

---

## Dev environment (Windows)

```sh
# Required environment variables
FFMPEG_DIR     = C:\ffmpeg
LIBCLANG_PATH  = C:\Program Files\LLVM\bin

cargo check --workspace   # type-check all crates
cargo build --release -p mystudiolab
```

---

## License

[MIT](LICENSE)
