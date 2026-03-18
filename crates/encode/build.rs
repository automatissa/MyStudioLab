/// build.rs for the `encode` crate.
///
/// The heavy lifting (finding FFmpeg headers, running bindgen) is done by
/// the `ffmpeg-next` crate's own build script.  This file exists so we can
/// add linker search paths or print helpful diagnostics if FFMPEG_DIR is
/// not set.
fn main() {
    if std::env::var("FFMPEG_DIR").is_err() {
        println!(
            "cargo:warning=FFMPEG_DIR is not set. \
             Set it to the root of your FFmpeg installation (e.g. C:\\ffmpeg) \
             so that ffmpeg-next can find the headers and libraries."
        );
    }

    if std::env::var("LIBCLANG_PATH").is_err() {
        println!(
            "cargo:warning=LIBCLANG_PATH is not set. \
             bindgen needs libclang to generate FFmpeg bindings. \
             Install LLVM and set LIBCLANG_PATH to the bin directory."
        );
    }
}
