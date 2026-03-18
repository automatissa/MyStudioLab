use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::info;

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

/// MyStudioLab — free, offline, open-source screen recorder with auto-zoom.
#[derive(Parser, Debug)]
#[command(name = "mystudiolab", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start a new recording session. Press Ctrl-C to stop.
    Record {
        /// Output file path (extension sets the container: .mp4 = H.265, .webm = AV1).
        #[arg(short, long, default_value = "recording.mp4")]
        output: PathBuf,

        /// Display index to capture (0 = primary).
        #[arg(short, long, default_value_t = 0)]
        display: usize,

        /// Frames per second.
        #[arg(long, default_value_t = 60)]
        fps: u32,

        /// Maximum zoom level (e.g. 2.0 = 2× zoom). 1.0 = no zoom.
        #[arg(long, default_value_t = 2.0)]
        zoom: f64,

        /// Disable auto-zoom entirely.
        #[arg(long)]
        no_zoom: bool,

        /// Capture microphone audio.
        #[arg(long)]
        mic: bool,

        /// Capture system audio (loopback).
        #[arg(long)]
        system_audio: bool,
    },

    /// List available displays.
    ListDisplays,

    /// List available audio input devices.
    ListAudioDevices,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "mystudiolab=info,capture=info,zoom=info,encode=info,audio=info".into()
            }),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Record {
            output,
            display,
            fps,
            zoom,
            no_zoom,
            mic,
            system_audio,
        } => {
            cmd_record(output, display, fps, zoom, no_zoom, mic, system_audio).await?;
        }
        Commands::ListDisplays => cmd_list_displays()?,
        Commands::ListAudioDevices => cmd_list_audio_devices()?,
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// `record` command
// ---------------------------------------------------------------------------

async fn cmd_record(
    output: PathBuf,
    display: usize,
    fps: u32,
    max_zoom: f64,
    no_zoom: bool,
    capture_mic: bool,
    capture_system: bool,
) -> Result<()> {
    // ---- 1. Start capture ------------------------------------------------
    let capture_config = capture::CaptureConfig {
        target: capture::CaptureTarget::Display(display),
        fps,
        show_cursor: true,
    };
    let mut recorder = capture::Recorder::new(capture_config);
    let frame_rx = recorder.start().context("Failed to start screen capture")?;

    // Block on the very first frame to discover the real capture resolution.
    // This drives the encoder's width/height so ffmpeg gets the right -video_size.
    let first_frame = frame_rx
        .recv()
        .context("Capture ended before producing any frames")?;
    let (cap_width, cap_height) = (first_frame.width, first_frame.height);
    info!(cap_width, cap_height, "Capture resolution detected");

    // ---- 2. Mouse tracker ------------------------------------------------
    let (mut mouse_tracker, mouse_rx) =
        zoom::MouseTracker::start().context("Failed to start mouse tracker")?;

    // ---- 3. Encoder ------------------------------------------------------
    let encode_config = encode::EncodeConfig {
        output_path: output.clone(),
        fps,
        width: cap_width,
        height: cap_height,
        ..Default::default()
    };
    let mut encoder = encode::Encoder::new(encode_config).context("Failed to start encoder")?;

    // ---- 4. Zoom processor -----------------------------------------------
    let zoom_config = zoom::ZoomConfig {
        max_zoom: if no_zoom { 1.0 } else { max_zoom },
        ..Default::default()
    };
    let mut zoom_proc = zoom::ZoomProcessor::new(zoom_config);

    // ---- 5. Audio (stub) -------------------------------------------------
    let audio_config = audio::AudioConfig {
        capture_mic,
        capture_system,
        ..Default::default()
    };
    let (mut audio_capturer, _audio_rx) =
        audio::AudioCapturer::start(audio_config).context("Failed to start audio capturer")?;

    // ---- 6. Pipeline thread ----------------------------------------------
    // The frame-receive loop is blocking, so it lives on a dedicated OS thread.
    // The thread returns the Encoder so we can call finish() after it exits.
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_thread = Arc::clone(&stop_flag);

    let pipeline = thread::Builder::new()
        .name("msl-pipeline".into())
        .spawn(move || -> Result<encode::Encoder> {
            let mut last_time = Instant::now();

            // Process the first frame (already received above).
            drain_mouse_events(&mouse_rx, &mut zoom_proc);
            let zoomed = zoom_proc
                .process(&first_frame, 0.0)
                .context("Zoom processing failed on first frame")?;
            encoder
                .encode_frame(&zoomed_to_raw(&zoomed))
                .context("Encode failed on first frame")?;

            loop {
                if stop_flag_thread.load(Ordering::Relaxed) {
                    break;
                }

                // Drain all pending mouse events before handling the next frame.
                drain_mouse_events(&mouse_rx, &mut zoom_proc);

                // Wait for the next captured frame (blocks until one arrives
                // or the sender is dropped — i.e. recorder.stop() is called).
                let frame = match frame_rx.recv() {
                    Ok(f) => f,
                    Err(_) => break, // channel closed — graceful shutdown
                };

                let delta_s = last_time.elapsed().as_secs_f64();
                last_time = Instant::now();

                let zoomed = zoom_proc
                    .process(&frame, delta_s)
                    .context("Zoom processing failed")?;

                encoder
                    .encode_frame(&zoomed_to_raw(&zoomed))
                    .context("Encode failed")?;
            }

            Ok(encoder)
        })
        .context("Failed to spawn pipeline thread")?;

    // ---- 7. Wait for Ctrl-C ----------------------------------------------
    info!("Recording… press Ctrl-C to stop  |  output: {}", output.display());
    tokio::signal::ctrl_c().await?;
    info!("Ctrl-C received — finishing recording");

    // ---- 8. Tear down ----------------------------------------------------
    // Signal the pipeline thread to exit, then close the frame sender by
    // stopping the recorder (drops the Sender end of frame_rx).
    stop_flag.store(true, Ordering::SeqCst);
    recorder.stop();    // closes frame channel → unblocks pipeline recv()
    mouse_tracker.stop();
    audio_capturer.stop();

    // Collect the encoder back from the pipeline thread and finalise the file.
    let encoder = pipeline
        .join()
        .map_err(|_| anyhow::anyhow!("Pipeline thread panicked"))?
        .context("Pipeline error")?;

    encoder.finish().context("Failed to finalise video file")?;

    info!("Saved → {}", output.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a [`zoom::ZoomedFrame`] into a [`capture::RawFrame`] for the encoder.
///
/// Both types are BGRA, same dimensions — this is a zero-copy-except-clone
/// adaptation; the zoom crate owns its buffer independently of the capture crate.
fn zoomed_to_raw(z: &zoom::ZoomedFrame) -> capture::RawFrame {
    capture::RawFrame {
        data: z.data.clone(),
        width: z.width,
        height: z.height,
        stride: z.width * 4,
        timestamp_us: z.timestamp_us,
    }
}

/// Drain all queued mouse events into the zoom processor.
fn drain_mouse_events(
    rx: &crossbeam_channel::Receiver<zoom::tracker::MouseEvent>,
    proc: &mut zoom::ZoomProcessor,
) {
    while let Ok(ev) = rx.try_recv() {
        proc.handle_mouse_event(&ev);
    }
}

// ---------------------------------------------------------------------------
// `list-displays` / `list-audio-devices`
// ---------------------------------------------------------------------------

fn cmd_list_displays() -> Result<()> {
    println!("Display enumeration not yet implemented.");
    Ok(())
}

fn cmd_list_audio_devices() -> Result<()> {
    println!("Audio device enumeration not yet implemented.");
    Ok(())
}
