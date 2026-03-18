use capture::RawFrame;
use tracing::{debug, trace};

use crate::{
    easing::{ease_in_out_cubic, lerp},
    tracker::MouseEvent,
    ZoomError,
};

/// A processed frame after the zoom crop/upscale has been applied.
#[derive(Debug, Clone)]
pub struct ZoomedFrame {
    /// Pixel data (BGRA, 4 bytes per pixel), upscaled to the source resolution.
    pub data: Vec<u8>,
    /// Width of the output frame (matches source capture width).
    pub width: u32,
    /// Height of the output frame (matches source capture height).
    pub height: u32,
    /// Timestamp forwarded from the source [`RawFrame`].
    pub timestamp_us: u64,
}

/// Configures the zoom behaviour.
#[derive(Debug, Clone)]
pub struct ZoomConfig {
    /// Maximum zoom level (e.g. `2.0` = 2× zoom, showing ½ the screen area).
    pub max_zoom: f64,
    /// Duration of the zoom-in transition in seconds.
    pub zoom_in_duration_s: f64,
    /// Duration of the zoom-out transition in seconds.
    pub zoom_out_duration_s: f64,
    /// How long to stay zoomed in after the last click before zooming out.
    pub hold_duration_s: f64,
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            max_zoom: 2.0,
            zoom_in_duration_s: 0.3,
            zoom_out_duration_s: 0.6,
            hold_duration_s: 1.5,
        }
    }
}

/// Internal zoom state machine.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ZoomState {
    /// Not zoomed; waiting for a click.
    Idle,
    /// Transitioning from current zoom level toward max zoom.
    ZoomingIn { progress: f64 },
    /// Holding at max zoom after a click.
    Holding { elapsed_s: f64 },
    /// Transitioning back toward no zoom.
    ZoomingOut { progress: f64 },
}

/// Stateful processor: consumes [`RawFrame`]s + [`MouseEvent`]s, produces
/// [`ZoomedFrame`]s.
pub struct ZoomProcessor {
    config: ZoomConfig,
    state: ZoomState,
    /// Current zoom level ∈ [1.0, config.max_zoom].
    current_zoom: f64,
    /// Current viewport centre in normalised screen coordinates [0.0, 1.0].
    cursor_norm: (f64, f64),
    /// Source frame dimensions (set on first frame).
    frame_size: Option<(u32, u32)>,
}

impl ZoomProcessor {
    pub fn new(config: ZoomConfig) -> Self {
        Self {
            config,
            state: ZoomState::Idle,
            current_zoom: 1.0,
            cursor_norm: (0.5, 0.5),
            frame_size: None,
        }
    }

    /// Feed a mouse event to update zoom state.
    pub fn handle_mouse_event(&mut self, event: &MouseEvent) {
        match event {
            MouseEvent::Move { x, y } => {
                if let Some((w, h)) = self.frame_size {
                    self.cursor_norm = (x / w as f64, y / h as f64);
                }
                trace!(cx = self.cursor_norm.0, cy = self.cursor_norm.1, "Cursor moved");
            }
            MouseEvent::ButtonPress { x, y } => {
                if let Some((w, h)) = self.frame_size {
                    self.cursor_norm = (x / w as f64, y / h as f64);
                }
                debug!("Click — triggering zoom-in");
                self.state = ZoomState::ZoomingIn { progress: 0.0 };
            }
            MouseEvent::ButtonRelease { .. } => {}
        }
    }

    /// Process one raw frame, advancing zoom state by `delta_s` seconds.
    ///
    /// Returns a [`ZoomedFrame`] at the original capture resolution.
    pub fn process(&mut self, frame: &RawFrame, delta_s: f64) -> Result<ZoomedFrame, ZoomError> {
        // Cache dimensions on first frame.
        if self.frame_size.is_none() {
            self.frame_size = Some((frame.width, frame.height));
        }

        // Advance the state machine.
        self.advance_state(delta_s);

        if (self.current_zoom - 1.0).abs() < 1e-4 {
            // No zoom — pass through with a straight copy.
            return Ok(ZoomedFrame {
                data: frame.data.clone(),
                width: frame.width,
                height: frame.height,
                timestamp_us: frame.timestamp_us,
            });
        }

        // Compute the crop rectangle.
        let out_w = frame.width as f64 / self.current_zoom;
        let out_h = frame.height as f64 / self.current_zoom;

        let cx = (self.cursor_norm.0 * frame.width as f64).clamp(out_w / 2.0, frame.width as f64 - out_w / 2.0);
        let cy = (self.cursor_norm.1 * frame.height as f64).clamp(out_h / 2.0, frame.height as f64 - out_h / 2.0);

        let src_x = (cx - out_w / 2.0).round() as u32;
        let src_y = (cy - out_h / 2.0).round() as u32;
        let crop_w = out_w.round() as u32;
        let crop_h = out_h.round() as u32;

        // TODO: replace this nearest-neighbour stub with a proper bicubic/lanczos upscaler.
        let upscaled = nearest_neighbour_upscale(
            &frame.data,
            frame.width,
            frame.height,
            frame.stride,
            src_x,
            src_y,
            crop_w,
            crop_h,
            frame.width,
            frame.height,
        );

        Ok(ZoomedFrame {
            data: upscaled,
            width: frame.width,
            height: frame.height,
            timestamp_us: frame.timestamp_us,
        })
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    fn advance_state(&mut self, delta_s: f64) {
        match self.state {
            ZoomState::Idle => {
                self.current_zoom = 1.0;
            }
            ZoomState::ZoomingIn { ref mut progress } => {
                *progress = (*progress + delta_s / self.config.zoom_in_duration_s).min(1.0);
                let t = ease_in_out_cubic(*progress);
                self.current_zoom = lerp(1.0, self.config.max_zoom, t);
                if *progress >= 1.0 {
                    self.state = ZoomState::Holding { elapsed_s: 0.0 };
                }
            }
            ZoomState::Holding { ref mut elapsed_s } => {
                *elapsed_s += delta_s;
                self.current_zoom = self.config.max_zoom;
                if *elapsed_s >= self.config.hold_duration_s {
                    self.state = ZoomState::ZoomingOut { progress: 0.0 };
                }
            }
            ZoomState::ZoomingOut { ref mut progress } => {
                *progress = (*progress + delta_s / self.config.zoom_out_duration_s).min(1.0);
                let t = ease_in_out_cubic(*progress);
                self.current_zoom = lerp(self.config.max_zoom, 1.0, t);
                if *progress >= 1.0 {
                    self.state = ZoomState::Idle;
                }
            }
        }
    }
}

/// Very simple nearest-neighbour crop + upscale.
///
/// This is a placeholder; swap with a high-quality resampler later.
fn nearest_neighbour_upscale(
    src: &[u8],
    _src_w: u32,
    _src_h: u32,
    src_stride: u32,
    crop_x: u32,
    crop_y: u32,
    crop_w: u32,
    crop_h: u32,
    dst_w: u32,
    dst_h: u32,
) -> Vec<u8> {
    let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = crop_x + (dx as f64 * crop_w as f64 / dst_w as f64) as u32;
            let sy = crop_y + (dy as f64 * crop_h as f64 / dst_h as f64) as u32;
            let src_off = (sy * src_stride + sx * 4) as usize;
            let dst_off = (dy * dst_w * 4 + dx * 4) as usize;
            if src_off + 4 <= src.len() && dst_off + 4 <= dst.len() {
                dst[dst_off..dst_off + 4].copy_from_slice(&src[src_off..src_off + 4]);
            }
        }
    }
    dst
}
