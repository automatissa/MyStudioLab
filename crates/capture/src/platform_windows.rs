/// Windows-specific capture backend using the Windows.Graphics.Capture API
/// via the `windows-capture` crate.
///
/// Architecture
/// ------------
/// `WinCaptureHandler` implements `GraphicsCaptureApiHandler`.
/// Its `on_frame_arrived` callback converts each `windows_capture::Frame`
/// into our `RawFrame` and sends it over a `crossbeam_channel::Sender`.
/// A stop flag (`Arc<AtomicBool>`) lets `Recorder::stop()` signal the handler
/// to call `capture_control.stop()` on the next callback tick.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use crossbeam_channel::Sender;
use tracing::{debug, warn};
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
};

use crate::{CaptureError, RawFrame};

// ------------------------------------------------------------------
// Flags passed to the handler constructor
// ------------------------------------------------------------------

/// Data forwarded from `Recorder::start()` to `WinCaptureHandler::new()`.
pub(crate) struct WinCaptureFlags {
    pub tx: Sender<RawFrame>,
    pub stop_flag: Arc<AtomicBool>,
    pub start_us: u64,
}

// ------------------------------------------------------------------
// Handler implementation
// ------------------------------------------------------------------

pub(crate) struct WinCaptureHandler {
    tx: Sender<RawFrame>,
    stop_flag: Arc<AtomicBool>,
    start_us: u64,
}

impl GraphicsCaptureApiHandler for WinCaptureHandler {
    type Flags = WinCaptureFlags;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self {
            tx: ctx.flags.tx,
            stop_flag: ctx.flags.stop_flag,
            start_us: ctx.flags.start_us,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        // Check for external stop signal.
        if self.stop_flag.load(Ordering::Relaxed) {
            capture_control.stop();
            return Ok(());
        }

        // Get BGRA pixels without row padding.
        let mut buffer = match frame.buffer() {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to get frame buffer: {e}");
                return Ok(());
            }
        };

        let width = buffer.width();
        let height = buffer.height();

        let data = match buffer.as_nopadding_buffer() {
            Ok(bytes) => bytes.to_vec(),
            Err(e) => {
                warn!("Failed to remove frame padding: {e}");
                return Ok(());
            }
        };

        let now_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;
        let timestamp_us = now_us.saturating_sub(self.start_us);

        let raw = RawFrame {
            stride: width * 4,
            data,
            width,
            height,
            timestamp_us,
        };

        if self.tx.send(raw).is_err() {
            // Receiver dropped — stop the capture.
            debug!("Frame receiver dropped; stopping capture");
            capture_control.stop();
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        debug!("Windows capture session closed");
        Ok(())
    }
}

// ------------------------------------------------------------------
// Builder helper: create Settings from CaptureConfig
// ------------------------------------------------------------------

pub(crate) fn build_settings(
    display_index: usize,
    _fps: u32,
    show_cursor: bool,
    flags: WinCaptureFlags,
) -> Result<Settings<WinCaptureFlags, Monitor>, CaptureError> {
    let monitors = Monitor::enumerate()
        .map_err(|e| CaptureError::BackendInit(format!("Monitor::enumerate failed: {e}")))?;

    if monitors.is_empty() {
        return Err(CaptureError::NoTargetFound);
    }

    let monitor = monitors
        .into_iter()
        .nth(display_index)
        .ok_or(CaptureError::NoTargetFound)?;

    let cursor = if show_cursor {
        CursorCaptureSettings::WithCursor
    } else {
        CursorCaptureSettings::WithoutCursor
    };

    let settings = Settings::new(
        monitor,
        cursor,
        DrawBorderSettings::WithoutBorder,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Bgra8,
        flags,
    );

    Ok(settings)
}

/// Call once at startup to set per-monitor DPI awareness (PMv2).
///
/// Ensures captured frame dimensions match physical pixels.
pub fn set_dpi_aware() {
    // TODO: call SetProcessDpiAwarenessContext via windows-sys
    tracing::debug!("DPI awareness (PMv2): stub — not yet applied");
}
