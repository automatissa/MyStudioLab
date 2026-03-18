//! # audio
//!
//! Audio capture (system loopback + microphone) with A/V sync.
//!
//! Responsibilities:
//! - Open mic and/or system-audio streams via `cpal`
//! - Emit interleaved PCM samples over a channel
//! - Track timestamps so the muxer can align audio with video frames
//! - On Windows: WASAPI loopback for system audio
//! - On macOS:   CoreAudio / BlackHole loopback
//! - On Linux:   PipeWire / PulseAudio monitor source

pub mod capturer;
pub mod error;

pub use capturer::{AudioCapturer, AudioConfig, AudioFrame};
pub use error::AudioError;
