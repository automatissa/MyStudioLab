use crossbeam_channel::{Receiver, Sender};
use tracing::{info, warn};

use crate::AudioError;

/// Which audio sources to capture.
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Capture microphone input.
    pub capture_mic: bool,
    /// Capture system audio (loopback).
    pub capture_system: bool,
    /// Sample rate in Hz (e.g. 48000).
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            capture_mic: true,
            capture_system: false, // opt-in; requires loopback device
            sample_rate: 48_000,
            channels: 2,
        }
    }
}

/// A chunk of interleaved PCM audio samples (f32).
#[derive(Debug, Clone)]
pub struct AudioFrame {
    /// Interleaved f32 samples: [L, R, L, R, …]
    pub samples: Vec<f32>,
    /// Monotonic timestamp in microseconds since recording started.
    pub timestamp_us: u64,
    /// Sample rate of this frame.
    pub sample_rate: u32,
    /// Channel count.
    pub channels: u16,
}

/// Manages audio capture sessions for mic and/or system loopback.
pub struct AudioCapturer {
    #[allow(dead_code)] // stored for future cpal stream configuration
    config: AudioConfig,
    // TODO: hold cpal streams
}

impl AudioCapturer {
    /// Initialise audio device(s) and return a channel receiver for
    /// [`AudioFrame`]s.
    pub fn start(config: AudioConfig) -> Result<(Self, Receiver<AudioFrame>), AudioError> {
        info!(
            mic = config.capture_mic,
            system = config.capture_system,
            sample_rate = config.sample_rate,
            "Starting audio capture"
        );

        let (_tx, rx): (Sender<AudioFrame>, Receiver<AudioFrame>) =
            crossbeam_channel::bounded(512);

        // TODO: open cpal input stream(s), push frames into _tx

        warn!("AudioCapturer::start is not yet implemented — no audio frames will be produced");

        Ok((Self { config }, rx))
    }

    /// Stop all audio streams.
    pub fn stop(&mut self) {
        info!("Stopping audio capture");
        // TODO: drop cpal streams
    }
}
