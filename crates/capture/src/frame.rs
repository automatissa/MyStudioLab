/// A single raw captured frame coming off the screen-capture backend.
///
/// Pixels are stored as BGRA / RGBA bytes depending on the platform;
/// downstream consumers must normalise to a common format before processing.
#[derive(Debug, Clone)]
pub struct RawFrame {
    /// Raw pixel data (BGRA or RGBA, 4 bytes per pixel).
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Stride (bytes per row). May be larger than `width * 4` due to padding.
    pub stride: u32,
    /// Monotonic timestamp in microseconds since the start of recording.
    pub timestamp_us: u64,
}

impl RawFrame {
    /// Returns the expected byte length for a tightly-packed BGRA frame.
    pub fn packed_len(&self) -> usize {
        (self.width * self.height * 4) as usize
    }
}
