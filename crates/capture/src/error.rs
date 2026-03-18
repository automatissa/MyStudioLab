use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("No capture target found")]
    NoTargetFound,

    #[error("Failed to initialise capture backend: {0}")]
    BackendInit(String),

    #[error("Frame capture failed: {0}")]
    FrameCapture(String),

    #[error("Permission denied — screen recording permission is required")]
    PermissionDenied,

    #[error("Capture channel closed unexpectedly")]
    ChannelClosed,

    #[error("Platform error: {0}")]
    Platform(String),
}
