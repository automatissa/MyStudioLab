use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crossbeam_channel::{Receiver, Sender};
use tracing::{debug, error, info, warn};

use crate::{CaptureError, RawFrame};

// ------------------------------------------------------------------
// Public types
// ------------------------------------------------------------------

/// Which part of the screen to record.
#[derive(Debug, Clone)]
pub enum CaptureTarget {
    /// Record a single display by index (0 = primary).
    Display(usize),
    /// Record the window whose title contains this substring.
    Window(String),
}

impl Default for CaptureTarget {
    fn default() -> Self {
        Self::Display(0)
    }
}

/// Configuration passed to [`Recorder::new`].
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub target: CaptureTarget,
    pub fps: u32,
    pub show_cursor: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            target: CaptureTarget::default(),
            fps: 60,
            show_cursor: true,
        }
    }
}

// ------------------------------------------------------------------
// Recorder
// ------------------------------------------------------------------

pub struct Recorder {
    config: CaptureConfig,
    stop_flag: Arc<AtomicBool>,
    /// Thread that either polls a CaptureControl (Windows) or runs the scap
    /// loop (macOS / Linux).
    capture_thread: Option<thread::JoinHandle<()>>,
}

impl Recorder {
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            stop_flag: Arc::new(AtomicBool::new(false)),
            capture_thread: None,
        }
    }

    pub fn start(&mut self) -> Result<Receiver<RawFrame>, CaptureError> {
        self.stop_flag.store(false, Ordering::SeqCst);
        let (tx, rx) = crossbeam_channel::bounded(128);
        let handle = self.start_platform(tx)?;
        self.capture_thread = Some(handle);
        Ok(rx)
    }

    pub fn stop(&mut self) {
        info!("Stopping screen capture");
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(h) = self.capture_thread.take() {
            let _ = h.join();
        }
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        self.stop();
    }
}

// ------------------------------------------------------------------
// Windows backend
// ------------------------------------------------------------------

#[cfg(target_os = "windows")]
impl Recorder {
    fn start_platform(&self, tx: Sender<RawFrame>) -> Result<thread::JoinHandle<()>, CaptureError> {
        use crate::platform_windows::{build_settings, WinCaptureFlags, WinCaptureHandler};
        use windows_capture::capture::GraphicsCaptureApiHandler;

        let display_index = match &self.config.target {
            CaptureTarget::Display(i) => *i,
            CaptureTarget::Window(_) => {
                // Window targeting is handled inside build_settings in a future iteration.
                // Fall back to display 0 for now.
                warn!("Window capture target not yet implemented on Windows; using display 0");
                0
            }
        };

        let start_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let flags = WinCaptureFlags {
            tx,
            stop_flag: Arc::clone(&self.stop_flag),
            start_us,
        };

        let settings = build_settings(display_index, self.config.fps, self.config.show_cursor, flags)?;
        let stop_flag = Arc::clone(&self.stop_flag);

        info!(display = display_index, fps = self.config.fps, "Starting Windows capture");

        let handle = thread::Builder::new()
            .name("msl-capture-ctrl".into())
            .spawn(move || {
                match WinCaptureHandler::start_free_threaded(settings) {
                    Ok(control) => {
                        // Poll until the stop flag is set or the session ends on its own,
                        // then call stop() which both signals the session and joins the
                        // capture thread.  stop() consumes the control so we must not
                        // call wait() afterwards.
                        loop {
                            if stop_flag.load(Ordering::Relaxed) || control.is_finished() {
                                if let Err(e) = control.stop() {
                                    error!("CaptureControl::stop error: {e}");
                                }
                                break;
                            }
                            thread::sleep(std::time::Duration::from_millis(50));
                        }
                    }
                    Err(e) => {
                        error!("windows-capture start_free_threaded failed: {e}");
                    }
                }
                debug!("Capture control thread exited");
            })
            .map_err(|e| CaptureError::Platform(format!("Failed to spawn capture thread: {e}")))?;

        Ok(handle)
    }
}

// ------------------------------------------------------------------
// macOS / Linux backend (scap)
// ------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
impl Recorder {
    fn start_platform(&self, tx: Sender<RawFrame>) -> Result<thread::JoinHandle<()>, CaptureError> {
        use scap::{
            capturer::{Capturer, Options, Resolution},
            frame::{Frame, FrameType, VideoFrame},
            Target,
        };

        if !scap::is_supported() {
            return Err(CaptureError::Platform(
                "Screen capture not supported on this platform".into(),
            ));
        }
        if !scap::has_permission() {
            scap::request_permission();
            if !scap::has_permission() {
                return Err(CaptureError::PermissionDenied);
            }
        }

        let scap_target = resolve_scap_target(&self.config.target)?;

        let options = Options {
            fps: self.config.fps,
            show_cursor: self.config.show_cursor,
            show_highlight: false,
            target: scap_target,
            crop_area: None,
            output_type: FrameType::BGRAFrame,
            output_resolution: Resolution::Captured,
            excluded_targets: None,
            captures_audio: false,
            exclude_current_process_audio: true,
        };

        let mut capturer = Capturer::build(options)
            .map_err(|e| CaptureError::BackendInit(format!("{e:?}")))?;

        let [width, height] = capturer.get_output_frame_size();
        info!(width, height, fps = self.config.fps, "Capture session opened (scap)");

        let stop_flag = Arc::clone(&self.stop_flag);

        let handle = thread::Builder::new()
            .name("msl-capture".into())
            .spawn(move || {
                capturer.start_capture();

                let start_us = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_micros() as u64;

                loop {
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    match capturer.get_next_frame() {
                        Ok(Frame::Video(vf)) => {
                            if let Some(raw) = scap_video_to_raw(vf, width, height, start_us) {
                                if tx.send(raw).is_err() {
                                    debug!("Frame receiver dropped — exiting scap capture loop");
                                    break;
                                }
                            }
                        }
                        Ok(Frame::Audio(_)) => {}
                        Err(e) => {
                            if stop_flag.load(Ordering::Relaxed) {
                                break;
                            }
                            error!("scap get_next_frame: {e}");
                            break;
                        }
                    }
                }
                capturer.stop_capture();
                debug!("scap capture loop exited");
            })
            .map_err(|e| CaptureError::Platform(format!("Failed to spawn capture thread: {e}")))?;

        Ok(handle)
    }
}

// ------------------------------------------------------------------
// scap target resolver (non-Windows)
// ------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
fn resolve_scap_target(target: &CaptureTarget) -> Result<Option<scap::Target>, CaptureError> {
    match target {
        CaptureTarget::Display(idx) => {
            let all = scap::get_all_targets();
            let displays: Vec<_> = all
                .into_iter()
                .filter(|t| matches!(t, scap::Target::Display(_)))
                .collect();
            if displays.is_empty() {
                return Err(CaptureError::NoTargetFound);
            }
            let picked = displays.into_iter().nth(*idx);
            if picked.is_none() {
                warn!(idx, "Display index out of range; falling back to primary");
            }
            Ok(picked)
        }
        CaptureTarget::Window(title_sub) => {
            let all = scap::get_all_targets();
            let found = all.into_iter().find(|t| {
                if let scap::Target::Window(w) = t {
                    w.title.to_lowercase().contains(&title_sub.to_lowercase())
                } else {
                    false
                }
            });
            if found.is_none() {
                warn!(title = %title_sub, "Window not found; falling back to primary display");
            }
            Ok(found)
        }
    }
}

// ------------------------------------------------------------------
// scap VideoFrame → RawFrame (non-Windows)
// ------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
fn scap_video_to_raw(
    frame: scap::frame::VideoFrame,
    fallback_w: u32,
    fallback_h: u32,
    start_us: u64,
) -> Option<RawFrame> {
    use scap::frame::VideoFrame;

    match frame {
        VideoFrame::BGRA(f) => {
            let ts = system_time_to_us(&f.display_time).saturating_sub(start_us);
            Some(RawFrame { stride: f.width * 4, data: f.data, width: f.width, height: f.height, timestamp_us: ts })
        }
        VideoFrame::BGRx(f) => {
            let ts = system_time_to_us(&f.display_time).saturating_sub(start_us);
            Some(RawFrame { stride: f.width * 4, data: f.data, width: f.width, height: f.height, timestamp_us: ts })
        }
        VideoFrame::RGB(f) => {
            let ts = system_time_to_us(&f.display_time).saturating_sub(start_us);
            Some(RawFrame { stride: f.width * 4, width: f.width, height: f.height, data: rgb_to_bgra(&f.data), timestamp_us: ts })
        }
        VideoFrame::YUVFrame(f) => {
            let ts = system_time_to_us(&f.display_time).saturating_sub(start_us);
            let data = yuv420_to_bgra(&f.luminance_bytes, &f.chrominance_bytes, f.width, f.height);
            Some(RawFrame { stride: f.width * 4, width: f.width, height: f.height, data, timestamp_us: ts })
        }
        _ => {
            warn!("Unsupported VideoFrame variant; emitting black frame");
            Some(RawFrame {
                data: vec![0u8; (fallback_w * fallback_h * 4) as usize],
                width: fallback_w, height: fallback_h,
                stride: fallback_w * 4, timestamp_us: 0,
            })
        }
    }
}

// ------------------------------------------------------------------
// Pixel-format helpers (non-Windows only — Windows always gives BGRA)
// ------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
fn rgb_to_bgra(src: &[u8]) -> Vec<u8> {
    let mut dst = Vec::with_capacity(src.len() / 3 * 4);
    for px in src.chunks_exact(3) {
        dst.push(px[2]); // B
        dst.push(px[1]); // G
        dst.push(px[0]); // R
        dst.push(255);   // A
    }
    dst
}

#[cfg(not(target_os = "windows"))]
fn yuv420_to_bgra(y_plane: &[u8], uv_plane: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let mut bgra = vec![0u8; w * h * 4];
    for row in 0..h {
        for col in 0..w {
            let y = y_plane[row * w + col] as f32;
            let uv_idx = (row / 2) * w + (col / 2) * 2;
            let u = uv_plane.get(uv_idx).copied().unwrap_or(128) as f32 - 128.0;
            let v = uv_plane.get(uv_idx + 1).copied().unwrap_or(128) as f32 - 128.0;
            let r = (y + 1.402 * v).clamp(0.0, 255.0) as u8;
            let g = (y - 0.344136 * u - 0.714136 * v).clamp(0.0, 255.0) as u8;
            let b = (y + 1.772 * u).clamp(0.0, 255.0) as u8;
            let off = (row * w + col) * 4;
            bgra[off] = b; bgra[off + 1] = g; bgra[off + 2] = r; bgra[off + 3] = 255;
        }
    }
    bgra
}

#[cfg(not(target_os = "windows"))]
fn system_time_to_us(t: &std::time::SystemTime) -> u64 {
    t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_micros() as u64
}
