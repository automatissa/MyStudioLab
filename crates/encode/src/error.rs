use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("FFmpeg error: {0}")]
    Ffmpeg(String),

    #[error("No suitable hardware encoder found; software fallback also unavailable")]
    NoEncoder,

    #[error("Failed to open output file '{path}': {reason}")]
    OutputFile { path: String, reason: String },

    #[error("Muxer error: {0}")]
    Muxer(String),

    #[error("Frame encode error: {0}")]
    Frame(String),
}
