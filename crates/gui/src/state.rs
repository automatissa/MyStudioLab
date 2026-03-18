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

// ---------------------------------------------------------------------------
// Recording settings (bound to UI widgets)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RecordSettings {
    pub output_path: String,
    pub display_index: usize,
    pub fps: u32,
    pub zoom_enabled: bool,
    pub max_zoom: f64,
    pub zoom_in_secs: f64,
    pub zoom_out_secs: f64,
    pub hold_secs: f64,
    pub capture_mic: bool,
    pub capture_system_audio: bool,
}

impl Default for RecordSettings {
    fn default() -> Self {
        // Default output path = ~/Videos/recording.mp4
        let home = dirs_next_path();
        Self {
            output_path: home,
            display_index: 0,
            fps: 60,
            zoom_enabled: true,
            max_zoom: 2.0,
            zoom_in_secs: 0.3,
            zoom_out_secs: 0.6,
            hold_secs: 1.5,
            capture_mic: false,
            capture_system_audio: false,
        }
    }
}

fn dirs_next_path() -> String {
    // Try $USERPROFILE\Videos or $HOME/Videos, fall back to current dir.
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        let p = PathBuf::from(home).join("Videos").join("recording.mp4");
        return p.to_string_lossy().into_owned();
    }
    "recording.mp4".into()
}

// ---------------------------------------------------------------------------
// Recording session (active while recording)
// ---------------------------------------------------------------------------

pub struct RecordingSession {
    pub stop_flag: Arc<AtomicBool>,
    pub pipeline_thread: Option<thread::JoinHandle<anyhow::Result<encode::Encoder>>>,
    pub started_at: Instant,
    pub frame_count: u64,
    pub recorder: capture::Recorder,
    pub mouse_tracker: zoom::MouseTracker,
    pub audio_capturer: audio::AudioCapturer,
}

// ---------------------------------------------------------------------------
// App-level UI state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Home,
    Settings,
}

pub enum RecordingState {
    Idle,
    Starting,
    Recording(Box<RecordingSession>),
    Finishing,
}

impl std::fmt::Debug for RecordingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingState::Idle => write!(f, "Idle"),
            RecordingState::Starting => write!(f, "Starting"),
            RecordingState::Recording(_) => write!(f, "Recording"),
            RecordingState::Finishing => write!(f, "Finishing"),
        }
    }
}

pub struct AppState {
    pub screen: AppScreen,
    pub settings: RecordSettings,
    pub recording: RecordingState,
    /// Status message shown in the status bar at the bottom.
    pub status: String,
    /// Non-empty if the last action produced an error.
    pub last_error: Option<String>,
    /// Elapsed recording time, updated every repaint.
    pub elapsed: Duration,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: AppScreen::Home,
            settings: RecordSettings::default(),
            recording: RecordingState::Idle,
            status: "Ready".into(),
            last_error: None,
            elapsed: Duration::ZERO,
        }
    }
}

impl AppState {
    /// Start a recording session.  Returns an error string on failure.
    pub fn start_recording(&mut self) -> Result<(), String> {
        // --- Capture ---
        let cfg = capture::CaptureConfig {
            target: capture::CaptureTarget::Display(self.settings.display_index),
            fps: self.settings.fps,
            show_cursor: true,
        };
        let mut recorder = capture::Recorder::new(cfg);
        let frame_rx = recorder
            .start()
            .map_err(|e| format!("Capture failed: {e}"))?;

        // Wait for first frame to get real dimensions.
        let first_frame = frame_rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "Capture timed out — no frames received".to_string())?;

        let (w, h) = (first_frame.width, first_frame.height);
        info!(w, h, "Capture resolution detected");

        // --- Mouse tracker ---
        let (mouse_tracker, mouse_rx) = zoom::MouseTracker::start()
            .map_err(|e| format!("Mouse tracker failed: {e}"))?;

        // --- Encoder ---
        let enc_cfg = encode::EncodeConfig {
            output_path: PathBuf::from(&self.settings.output_path),
            fps: self.settings.fps,
            width: w,
            height: h,
            ..Default::default()
        };
        let mut encoder =
            encode::Encoder::new(enc_cfg).map_err(|e| format!("Encoder failed: {e}"))?;

        // --- Zoom ---
        let zoom_cfg = zoom::ZoomConfig {
            max_zoom: if self.settings.zoom_enabled {
                self.settings.max_zoom
            } else {
                1.0
            },
            zoom_in_duration_s: self.settings.zoom_in_secs,
            zoom_out_duration_s: self.settings.zoom_out_secs,
            hold_duration_s: self.settings.hold_secs,
        };
        let mut zoom_proc = zoom::ZoomProcessor::new(zoom_cfg);

        // --- Audio ---
        let audio_cfg = audio::AudioConfig {
            capture_mic: self.settings.capture_mic,
            capture_system: self.settings.capture_system_audio,
            ..Default::default()
        };
        let (audio_capturer, _audio_rx) = audio::AudioCapturer::start(audio_cfg)
            .map_err(|e| format!("Audio failed: {e}"))?;

        // --- Pipeline thread ---
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop_flag);

        let pipeline_thread = thread::Builder::new()
            .name("msl-pipeline".into())
            .spawn(move || -> anyhow::Result<encode::Encoder> {
                use std::time::Instant;
                let mut last = Instant::now();

                // Process first frame (already received above).
                drain_mouse(&mouse_rx, &mut zoom_proc);
                let zoomed = zoom_proc.process(&first_frame, 0.0)?;
                encoder.encode_frame(&zoomed_to_raw(&zoomed))?;

                loop {
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }
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
            .map_err(|e| format!("Failed to spawn pipeline: {e}"))?;

        self.recording = RecordingState::Recording(Box::new(RecordingSession {
            stop_flag,
            pipeline_thread: Some(pipeline_thread),
            started_at: Instant::now(),
            frame_count: 0,
            recorder,
            mouse_tracker,
            audio_capturer,
        }));

        self.status = "Recording…".into();
        self.last_error = None;
        self.elapsed = Duration::ZERO;

        Ok(())
    }

    /// Stop the active recording session and write the file.
    pub fn stop_recording(&mut self) {
        let prev = std::mem::replace(&mut self.recording, RecordingState::Finishing);

        if let RecordingState::Recording(mut session) = prev {
            session.stop_flag.store(true, Ordering::SeqCst);
            session.recorder.stop();
            session.mouse_tracker.stop();
            session.audio_capturer.stop();

            if let Some(handle) = session.pipeline_thread.take() {
                match handle.join() {
                    Ok(Ok(encoder)) => {
                        if let Err(e) = encoder.finish() {
                            self.last_error =
                                Some(format!("Failed to write output file: {e}"));
                            error!("{}", self.last_error.as_ref().unwrap());
                        } else {
                            self.status =
                                format!("Saved → {}", self.settings.output_path);
                        }
                    }
                    Ok(Err(e)) => {
                        self.last_error = Some(format!("Pipeline error: {e}"));
                        error!("{}", self.last_error.as_ref().unwrap());
                    }
                    Err(_) => {
                        self.last_error = Some("Pipeline thread panicked".into());
                        error!("Pipeline thread panicked");
                    }
                }
            }
        }

        self.recording = RecordingState::Idle;
        if self.last_error.is_none() && !self.status.starts_with("Saved") {
            self.status = "Ready".into();
        }
    }

    pub fn is_recording(&self) -> bool {
        matches!(self.recording, RecordingState::Recording(_))
    }

    /// Call once per frame to update the elapsed timer.
    pub fn tick(&mut self) {
        if let RecordingState::Recording(s) = &self.recording {
            self.elapsed = s.started_at.elapsed();
        }
    }
}

// ---------------------------------------------------------------------------
// Pipeline helpers
// ---------------------------------------------------------------------------

fn drain_mouse(
    rx: &crossbeam_channel::Receiver<zoom::tracker::MouseEvent>,
    proc: &mut zoom::ZoomProcessor,
) {
    while let Ok(ev) = rx.try_recv() {
        proc.handle_mouse_event(&ev);
    }
}

fn zoomed_to_raw(z: &zoom::ZoomedFrame) -> RawFrame {
    RawFrame {
        data: z.data.clone(),
        width: z.width,
        height: z.height,
        stride: z.width * 4,
        timestamp_us: z.timestamp_us,
    }
}
