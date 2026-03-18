//! # zoom
//!
//! Auto-zoom logic and frame processing.
//!
//! Responsibilities:
//! - Track mouse position and click events via `rdev`
//! - Decide when to zoom in/out based on click activity
//! - Compute the zoom viewport (a rectangle centred on the cursor)
//! - Smoothly interpolate the viewport using an easing function
//! - Crop + upscale each [`RawFrame`] to produce a [`ZoomedFrame`]

pub mod easing;
pub mod error;
pub mod processor;
pub mod tracker;

pub use error::ZoomError;
pub use processor::{ZoomConfig, ZoomProcessor, ZoomedFrame};
pub use tracker::MouseTracker;
