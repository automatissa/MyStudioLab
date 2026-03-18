use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use tracing::{error, info};

use capture::RawFrame;

// ── Recording settings ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RecordSettings {
    pub output_path:          String,
    pub display_index:        usize,
    pub fps:                  u32,
    pub zoom_enabled:         bool,
    pub max_zoom:             f64,
    pub zoom_in_secs:         f64,
    pub zoom_out_secs:        f64,
    pub hold_secs:            f64,
    pub capture_mic:          bool,
    pub capture_system_audio: bool,
}

impl Default for RecordSettings {
    fn default() -> Self {
        Self {
            output_path:          default_output_path(),
            display_index:        0,
            fps:                  60,
            zoom_enabled:         true,   // on by default — the key feature
            max_zoom:             2.0,
            zoom_in_secs:         0.3,
            zoom_out_secs:        0.6,
            hold_secs:            1.5,
            capture_mic:          false,
            capture_system_audio: false,
        }
    }
}

/// Auto-generates a timestamped filename, e.g.
/// `C:\Users\…\Videos\Recording 2024-01-15 14.30.mp4`
fn default_output_path() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple date from epoch seconds (avoids pulling in chrono)
    let (y, mo, d, h, m) = epoch_to_ymdh(now);
    let name = format!("Recording {y:04}-{mo:02}-{d:02} {h:02}.{m:02}.mp4");

    let videos = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(|home| PathBuf::from(home).join("Videos"))
        .unwrap_or_else(|_| PathBuf::from("."));

    videos.join(name).to_string_lossy().into_owned()
}

fn epoch_to_ymdh(secs: u64) -> (u32, u32, u32, u32, u32) {
    let s = secs % 60;
    let _ = s;
    let mins  = secs / 60;
    let hours = mins / 60;
    let days  = hours / 24;
    let h     = (hours % 24) as u32;
    let m     = (mins  % 60) as u32;

    // Days since 1970-01-01
    let mut y = 1970u32;
    let mut rem = days;
    loop {
        let leap = if y % 400 == 0 { 366 } else if y % 100 == 0 { 365 } else if y % 4 == 0 { 366 } else { 365 };
        if rem < leap { break; }
        rem -= leap;
        y += 1;
    }
    let leap_year = y % 400 == 0 || (y % 4 == 0 && y % 100 != 0);
    let days_in_month = [31u64, if leap_year { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mo = 0usize;
    while mo < 12 && rem >= days_in_month[mo] {
        rem -= days_in_month[mo];
        mo += 1;
    }
    (y, mo as u32 + 1, rem as u32 + 1, h, m)
}

// ── Recording session ─────────────────────────────────────────────────────────

pub struct RecordingSession {
    pub stop_flag:       Arc<AtomicBool>,
    pub pipeline_thread: Option<thread::JoinHandle<anyhow::Result<encode::Encoder>>>,
    pub started_at:      Instant,
    pub recorder:        capture::Recorder,
    pub mouse_tracker:   zoom::MouseTracker,
    pub audio_capturer:  audio::AudioCapturer,
}

// ── App screens ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Record,
    Settings,
}

pub enum RecordingState {
    Idle,
    Recording(Box<RecordingSession>),
    Finishing,
}

pub struct AppState {
    pub screen:     AppScreen,
    pub settings:   RecordSettings,
    pub recording:  RecordingState,
    pub status:     String,
    pub last_error: Option<String>,
    pub elapsed:    Duration,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen:     AppScreen::Record,
            settings:   RecordSettings::default(),
            recording:  RecordingState::Idle,
            status:     "Ready".into(),
            last_error: None,
            elapsed:    Duration::ZERO,
        }
    }
}

impl AppState {
    pub fn is_recording(&self) -> bool {
        matches!(self.recording, RecordingState::Recording(_))
    }

    pub fn tick(&mut self) {
        if let RecordingState::Recording(s) = &self.recording {
            self.elapsed = s.started_at.elapsed();
        }
    }

    pub fn start_recording(&mut self) -> Result<(), String> {
        let cfg = capture::CaptureConfig {
            target:      capture::CaptureTarget::Display(self.settings.display_index),
            fps:         self.settings.fps,
            show_cursor: true,
        };
        let mut recorder = capture::Recorder::new(cfg);
        let frame_rx = recorder.start().map_err(|e| format!("Capture: {e}"))?;

        let first_frame = frame_rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "No frames received — is another app blocking capture?".to_string())?;

        let (w, h) = (first_frame.width, first_frame.height);
        info!(w, h, "Capture resolution");

        let (mouse_tracker, mouse_rx) = zoom::MouseTracker::start()
            .map_err(|e| format!("Mouse tracker: {e}"))?;

        let enc_cfg = encode::EncodeConfig {
            output_path: PathBuf::from(&self.settings.output_path),
            fps:         self.settings.fps,
            width:       w,
            height:      h,
            ..Default::default()
        };
        let mut encoder = encode::Encoder::new(enc_cfg)
            .map_err(|e| format!("Encoder: {e}"))?;

        let zoom_cfg = zoom::ZoomConfig {
            max_zoom:          if self.settings.zoom_enabled { self.settings.max_zoom } else { 1.0 },
            zoom_in_duration_s:  self.settings.zoom_in_secs,
            zoom_out_duration_s: self.settings.zoom_out_secs,
            hold_duration_s:     self.settings.hold_secs,
        };
        let mut zoom_proc = zoom::ZoomProcessor::new(zoom_cfg);

        let audio_cfg = audio::AudioConfig {
            capture_mic:    self.settings.capture_mic,
            capture_system: self.settings.capture_system_audio,
            ..Default::default()
        };
        let (audio_capturer, _audio_rx) = audio::AudioCapturer::start(audio_cfg)
            .map_err(|e| format!("Audio: {e}"))?;

        let stop_flag  = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop_flag);

        let pipeline_thread = thread::Builder::new()
            .name("msl-pipeline".into())
            .spawn(move || -> anyhow::Result<encode::Encoder> {
                let mut last = Instant::now();

                drain_mouse(&mouse_rx, &mut zoom_proc);
                let zoomed = zoom_proc.process(&first_frame, 0.0)?;
                encoder.encode_frame(&zoomed_to_raw(&zoomed))?;

                loop {
                    if stop_clone.load(Ordering::Relaxed) { break; }
                    drain_mouse(&mouse_rx, &mut zoom_proc);
                    match frame_rx.recv() {
                        Ok(frame) => {
                            let dt = last.elapsed().as_secs_f64();
                            last = Instant::now();
                            let zoomed = zoom_proc.process(&frame, dt)?;
                            encoder.encode_frame(&zoomed_to_raw(&zoomed))?;
                        }
                        Err(_) => break,
                    }
                }
                Ok(encoder)
            })
            .map_err(|e| format!("Pipeline thread: {e}"))?;

        self.recording = RecordingState::Recording(Box::new(RecordingSession {
            stop_flag,
            pipeline_thread: Some(pipeline_thread),
            started_at: Instant::now(),
            recorder,
            mouse_tracker,
            audio_capturer,
        }));
        self.status     = "Recording…".into();
        self.last_error = None;
        self.elapsed    = Duration::ZERO;
        Ok(())
    }

    pub fn stop_recording(&mut self) {
        let prev = std::mem::replace(&mut self.recording, RecordingState::Finishing);
        if let RecordingState::Recording(mut session) = prev {
            session.stop_flag.store(true, Ordering::SeqCst);
            session.recorder.stop();
            session.mouse_tracker.stop();
            session.audio_capturer.stop();

            if let Some(handle) = session.pipeline_thread.take() {
                match handle.join() {
                    Ok(Ok(encoder)) => match encoder.finish() {
                        Ok(_)  => self.status = format!("Saved → {}", self.settings.output_path),
                        Err(e) => self.last_error = Some(format!("Failed to save: {e}")),
                    },
                    Ok(Err(e)) => self.last_error = Some(format!("Pipeline error: {e}")),
                    Err(_)     => self.last_error = Some("Pipeline thread panicked".into()),
                }
            }
        }
        self.recording = RecordingState::Idle;
        if self.last_error.is_some() { error!("{}", self.last_error.as_ref().unwrap()); }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn drain_mouse(
    rx: &crossbeam_channel::Receiver<zoom::tracker::MouseEvent>,
    proc: &mut zoom::ZoomProcessor,
) {
    while let Ok(ev) = rx.try_recv() { proc.handle_mouse_event(&ev); }
}

fn zoomed_to_raw(z: &zoom::ZoomedFrame) -> RawFrame {
    RawFrame { data: z.data.clone(), width: z.width, height: z.height,
               stride: z.width * 4, timestamp_us: z.timestamp_us }
}
