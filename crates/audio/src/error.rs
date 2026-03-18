use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Failed to initialise audio device: {0}")]
    DeviceInit(String),

    #[error("Audio stream error: {0}")]
    Stream(String),

    #[error("No audio input device found")]
    NoDevice,

    #[error("Audio channel closed")]
    ChannelClosed,
}
