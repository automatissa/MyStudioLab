//! # capture
//!
//! Cross-platform screen capture module.
//!
//! Responsibilities:
//! - Enumerate available displays / windows
//! - Open a capture session via the `scap` crate
//! - Emit a stream of [`RawFrame`]s over a [`crossbeam_channel`] channel
//! - Handle platform-specific quirks (DPI on Windows, permissions on macOS,
//!   PipeWire/X11 selection on Linux)

pub mod error;
pub mod frame;
pub mod recorder;

#[cfg(target_os = "windows")]
pub mod platform_windows;

#[cfg(target_os = "macos")]
pub mod platform_macos;

#[cfg(target_os = "linux")]
pub mod platform_linux;

pub use error::CaptureError;
pub use frame::RawFrame;
pub use recorder::{CaptureConfig, CaptureTarget, Recorder};
