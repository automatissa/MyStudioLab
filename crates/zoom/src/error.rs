use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZoomError {
    #[error("Mouse tracking failed to start: {0}")]
    TrackerInit(String),

    #[error("Frame processing failed: {0}")]
    FrameProcess(String),

    #[error("Invalid zoom configuration: {0}")]
    InvalidConfig(String),
}
