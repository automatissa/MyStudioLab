//! # encode
//!
//! Hardware-accelerated video encoding pipeline for MyStudioLab.
//!
//! ## How it works
//!
//! Instead of linking against the FFmpeg C library (which requires a matching
//! dev package and creates ABI headaches on Windows), this crate spawns the
//! `ffmpeg` binary as a child process and pipes raw BGRA frames into its
//! stdin.  FFmpeg handles pixel-format conversion, hardware encoding, and
//! container muxing internally.
//!
//! ```text
//! Recorder ──RawFrame(BGRA)──► Encoder::encode_frame()
//!                                   │
//!                              write to stdin
//!                                   │
//!                              ffmpeg process
//!                                   │
//!                              H.265/MP4 file  (or AV1/WebM)
//! ```
//!
//! ## Requirements
//!
//! Only the `ffmpeg` binary needs to be on `PATH`.  No dev package, no `.lib`
//! files, no bindgen — the full gyan.dev or system install is sufficient.
//!
//! ## Encoder auto-detection
//!
//! [`HwAccel::detect`] runs `ffmpeg -encoders` at startup and selects the
//! first available HEVC encoder in priority order:
//! `hevc_nvenc` → `hevc_amf` → `hevc_vaapi` → `hevc_videotoolbox` → `libx265`.

pub mod encoder;
pub mod error;
pub mod hw_accel;
pub mod muxer;

pub use encoder::{EncodeConfig, Encoder, OutputCodec};
pub use error::EncodeError;
pub use hw_accel::HwAccel;
