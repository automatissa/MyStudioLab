use std::{
    io::Write,
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
// CREATE_NO_WINDOW — prevents a console window flashing when ffmpeg is spawned
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

use capture::RawFrame;
use tracing::{debug, info, warn};

use crate::{EncodeError, HwAccel};

// ---------------------------------------------------------------------------
// Public config types
// ---------------------------------------------------------------------------

/// Output codec selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputCodec {
    /// H.265 / HEVC in MP4 container.
    #[default]
    H265,
    /// AV1 in WebM container.
    Av1,
}

impl OutputCodec {
    fn container_extension(&self) -> &'static str {
        match self {
            OutputCodec::H265 => "mp4",
            OutputCodec::Av1 => "webm",
        }
    }

    fn av1_encoder_name(&self) -> &'static str {
        // libaom-av1 is the most widely available fallback; svt-av1 is faster
        // if compiled in.
        "libaom-av1"
    }
}

/// Configuration passed to [`Encoder::new`].
#[derive(Debug, Clone)]
pub struct EncodeConfig {
    /// Path to the output video file (extension is forced to match the codec).
    pub output_path: PathBuf,
    /// Codec to use.
    pub codec: OutputCodec,
    /// Width of the encoded video in pixels.
    pub width: u32,
    /// Height of the encoded video in pixels.
    pub height: u32,
    /// Frames per second.
    pub fps: u32,
    /// Constant Rate Factor / quality level (lower = better quality, 0–51).
    pub crf: u8,
    /// Hardware backend to use; `None` means auto-detect.
    pub hw_accel: Option<HwAccel>,
}

impl Default for EncodeConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("recording.mp4"),
            codec: OutputCodec::H265,
            width: 1920,
            height: 1080,
            fps: 60,
            crf: 23,
            hw_accel: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Encoder
// ---------------------------------------------------------------------------

/// Manages an `ffmpeg` child process that encodes raw BGRA frames to a file.
///
/// ## Pipeline
/// ```text
/// RawFrame (BGRA) ──stdin──► ffmpeg ──► H.265/MP4 or AV1/WebM file
/// ```
///
/// No FFmpeg dev package or C bindings are required — only the `ffmpeg`
/// binary must be on `PATH` (or discoverable via the OS).
pub struct Encoder {
    config: EncodeConfig,
    #[allow(dead_code)] // stored for future inspection / serialisation
    hw_accel: HwAccel,
    /// `ffmpeg` child process.
    child: Child,
    /// Handle to the child's stdin where we write raw frame bytes.
    /// Wrapped in Option so `finish()` can take it while the type still
    /// implements `Drop`.
    stdin: Option<ChildStdin>,
    /// Count of frames encoded so far (for logging).
    frame_count: u64,
}

impl Encoder {
    /// Spawn the `ffmpeg` process and prepare it to receive frames.
    pub fn new(mut config: EncodeConfig) -> Result<Self, EncodeError> {
        let hw_accel = config.hw_accel.unwrap_or_else(HwAccel::detect);
        config.hw_accel = Some(hw_accel);

        // Force the output extension to match the container.
        config
            .output_path
            .set_extension(config.codec.container_extension());

        info!(
            path = %config.output_path.display(),
            encoder = %hw_accel,
            width = config.width,
            height = config.height,
            fps = config.fps,
            crf = config.crf,
            "Spawning ffmpeg encoder process"
        );

        let mut cmd = Self::build_command(&config, hw_accel);
        debug!("ffmpeg command: {:?}", cmd);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                EncodeError::Ffmpeg(format!(
                    "Failed to spawn ffmpeg (is it on PATH?): {e}"
                ))
            })?;

        let stdin = child.stdin.take().expect("piped stdin was not available");

        Ok(Self {
            config,
            hw_accel,
            child,
            stdin: Some(stdin),
            frame_count: 0,
        })
    }

    /// Write a single raw frame into the encoder.
    ///
    /// Frames must arrive in presentation order (monotonically increasing PTS).
    /// The frame data must be BGRA, `width × height × 4` bytes with no row padding.
    pub fn encode_frame(&mut self, frame: &RawFrame) -> Result<(), EncodeError> {
        // Validate dimensions match the encoder configuration.
        if frame.width != self.config.width || frame.height != self.config.height {
            warn!(
                frame_w = frame.width,
                frame_h = frame.height,
                enc_w = self.config.width,
                enc_h = self.config.height,
                "Frame size mismatch — dropping frame"
            );
            return Ok(());
        }

        match self.stdin.as_mut() {
            Some(s) => s
                .write_all(&frame.data)
                .map_err(|e| EncodeError::Frame(format!("stdin write failed: {e}")))?,
            None => return Err(EncodeError::Frame("encoder already finished".into())),
        };

        self.frame_count += 1;
        Ok(())
    }

    /// Flush, close stdin, and wait for `ffmpeg` to finish writing the file.
    ///
    /// Consumes the encoder; the output file is complete and safe to use
    /// after this returns.
    pub fn finish(mut self) -> Result<(), EncodeError> {
        info!(
            frames = self.frame_count,
            path = %self.config.output_path.display(),
            "Finalising video file"
        );

        // Closing stdin signals EOF to ffmpeg — it will flush and write the trailer.
        drop(self.stdin.take());

        let status = self
            .child
            .wait()
            .map_err(|e| EncodeError::Ffmpeg(format!("ffmpeg wait failed: {e}")))?;

        if !status.success() {
            return Err(EncodeError::Ffmpeg(format!(
                "ffmpeg exited with status {status}"
            )));
        }

        info!("ffmpeg finished successfully");
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn build_command(config: &EncodeConfig, hw_accel: HwAccel) -> Command {
        let mut cmd = Command::new("ffmpeg");

        // --- Input: raw BGRA video from stdin ---
        cmd.args([
            "-hide_banner",
            "-f",
            "rawvideo",
            "-pixel_format",
            "bgra",
            "-video_size",
            &format!("{}x{}", config.width, config.height),
            "-framerate",
            &config.fps.to_string(),
            "-i",
            "pipe:0", // read from stdin
        ]);

        // --- Codec ---
        let encoder_name = match config.codec {
            OutputCodec::H265 => hw_accel.hevc_encoder_name(),
            OutputCodec::Av1 => config.codec.av1_encoder_name(),
        };
        cmd.args(["-c:v", encoder_name]);

        // --- Quality / codec-specific options ---
        let extra = hw_accel.ffmpeg_args(config.crf);
        for (flag, value) in &extra {
            cmd.args([*flag, value.as_str()]);
        }

        // --- Output ---
        cmd.args([
            "-y", // overwrite output file without asking
            config.output_path.to_str().unwrap_or("recording.mp4"),
        ]);

        cmd
    }
}

impl Drop for Encoder {
    /// If the encoder is dropped without calling `finish()`, kill the ffmpeg
    /// process to avoid leaving a zombie and a corrupted output file.
    fn drop(&mut self) {
        // Close stdin first so ffmpeg isn't blocked waiting for more input.
        drop(self.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
