/// Linux-specific helpers.
///
/// Handles:
/// - Detecting whether the compositor is X11 or Wayland
/// - PipeWire portal setup for Wayland screen capture
/// - X11 RANDR monitor enumeration

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

/// Detect the display server in use at runtime.
pub fn detect_display_server() -> DisplayServer {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        DisplayServer::Wayland
    } else if std::env::var("DISPLAY").is_ok() {
        DisplayServer::X11
    } else {
        DisplayServer::Unknown
    }
}
