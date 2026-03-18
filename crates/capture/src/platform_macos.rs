/// macOS-specific helpers for the capture crate.
///
/// scap's ScreenCaptureKit backend calls `CGRequestScreenCaptureAccess`
/// internally, but we expose thin wrappers here so the rest of the codebase
/// can check / request permission without depending on scap directly.

/// Returns `true` if the app already has screen recording permission.
///
/// Delegates to `scap::has_permission()` which wraps
/// `CGPreflightScreenCaptureAccess`.
pub fn has_screen_capture_permission() -> bool {
    scap::has_permission()
}

/// Opens the Privacy & Security system pane so the user can grant access.
///
/// Delegates to `scap::request_permission()` which wraps
/// `CGRequestScreenCaptureAccess`.
pub fn request_screen_capture_permission() {
    scap::request_permission();
}
