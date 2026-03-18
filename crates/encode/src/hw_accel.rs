/// Hardware acceleration backend selection.
///
/// Detection works by running `ffmpeg -encoders` and scanning the output —
/// no C bindings or FFmpeg dev package required.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwAccel {
    /// NVIDIA NVENC (Windows / Linux)
    Nvenc,
    /// Apple VideoToolbox (macOS)
    VideoToolbox,
    /// Intel/AMD VA-API (Linux)
    Vaapi,
    /// AMD AMF (Windows)
    Amf,
    /// Pure software libx265 — last resort
    Software,
}

impl HwAccel {
    /// The FFmpeg encoder name for H.265/HEVC.
    pub fn hevc_encoder_name(&self) -> &'static str {
        match self {
            HwAccel::Nvenc => "hevc_nvenc",
            HwAccel::VideoToolbox => "hevc_videotoolbox",
            HwAccel::Vaapi => "hevc_vaapi",
            HwAccel::Amf => "hevc_amf",
            HwAccel::Software => "libx265",
        }
    }

    /// Probe the running `ffmpeg` binary and return the best available backend.
    ///
    /// Preference order: NVENC → AMF → VAAPI → VideoToolbox → libx265.
    /// Falls back to `Software` if the binary cannot be found or produces no
    /// recognised encoder names.
    pub fn detect() -> Self {
        let encoder_list = match Self::query_encoders() {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("HwAccel::detect — could not run ffmpeg: {e}");
                return HwAccel::Software;
            }
        };

        let candidates = [
            HwAccel::Nvenc,
            HwAccel::Amf,
            HwAccel::Vaapi,
            HwAccel::VideoToolbox,
            HwAccel::Software,
        ];

        for accel in candidates {
            if encoder_list.contains(accel.hevc_encoder_name()) {
                tracing::info!(encoder = accel.hevc_encoder_name(), "Selected hardware encoder");
                return accel;
            }
        }

        tracing::warn!("No HEVC encoder found in ffmpeg output — falling back to Software");
        HwAccel::Software
    }

    /// Returns the encoder options to append to the ffmpeg command line.
    ///
    /// Each element is a `("-flag", "value")` pair.
    pub(crate) fn ffmpeg_args(&self, crf: u8) -> Vec<(&'static str, String)> {
        match self {
            HwAccel::Software => vec![
                ("-crf", crf.to_string()),
                ("-preset", "medium".into()),
                // Suppress the x265 progress banner on stderr
                ("-x265-params", "log-level=error".into()),
            ],
            HwAccel::Nvenc => vec![
                ("-rc", "vbr".into()),
                ("-cq", crf.to_string()),
                ("-preset", "p4".into()),
                ("-tune", "hq".into()),
            ],
            HwAccel::Amf => vec![
                ("-quality", "balanced".into()),
                ("-rc", "cqp".into()),
                ("-qp_i", crf.to_string()),
                ("-qp_p", crf.to_string()),
            ],
            HwAccel::VideoToolbox => vec![
                ("-b:v", "8M".into()),
                ("-allow_sw", "1".into()),
            ],
            HwAccel::Vaapi => vec![
                ("-qp", crf.to_string()),
            ],
        }
    }

    // -------------------------------------------------------------------------

    fn query_encoders() -> std::io::Result<String> {
        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;
        #[cfg(target_os = "windows")]
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.args(["-hide_banner", "-encoders"]);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output()?;
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

impl std::fmt::Display for HwAccel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hevc_encoder_name())
    }
}
