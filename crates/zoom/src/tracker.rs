use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use crossbeam_channel::{Receiver, Sender};
use rdev::{listen, EventType};
use tracing::{debug, error, info};

use crate::ZoomError;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A mouse event relevant to the zoom controller.
#[derive(Debug, Clone)]
pub enum MouseEvent {
    /// Cursor moved to (x, y) in physical screen pixels.
    Move { x: f64, y: f64 },
    /// A mouse button was pressed.
    ButtonPress { x: f64, y: f64 },
    /// A mouse button was released.
    ButtonRelease { x: f64, y: f64 },
}

// ---------------------------------------------------------------------------
// MouseTracker
// ---------------------------------------------------------------------------

/// Listens to global mouse events via `rdev` and emits [`MouseEvent`]s
/// over a channel.
///
/// ## Threading model
///
/// `rdev::listen` is **blocking** and must run on its own OS thread.
/// The thread is spawned in `start()` and cleaned up in `stop()`.
/// Because `rdev` has no cancellation API, `stop()` sends a sentinel value
/// through a second `stop_tx` channel whose receiver is the listener thread.
pub struct MouseTracker {
    stop_flag: Arc<AtomicBool>,
    _listener_thread: thread::JoinHandle<()>,
}

impl MouseTracker {
    /// Spawn the rdev listener thread and return the event receiver.
    pub fn start() -> Result<(Self, Receiver<MouseEvent>), ZoomError> {
        info!("Starting mouse tracker");

        let (tx, rx): (Sender<MouseEvent>, Receiver<MouseEvent>) =
            crossbeam_channel::bounded(256);

        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = Arc::clone(&stop_flag);

        // rdev::listen blocks the calling thread forever, so we must spawn.
        //
        // NOTE: rdev has no cancellation support. When stop() is called we set
        // the flag so on_event stops forwarding, but the thread itself can only
        // exit when the next native OS event fires.  For a screen-recorder use
        // case (user is actively using the mouse) this is effectively instant.
        let handle = thread::Builder::new()
            .name("msl-mouse-tracker".into())
            .spawn(move || {
                let tx_inner = tx;
                let flag = stop_flag_clone;

                // Last known cursor position (used to attach coordinates to
                // ButtonPress/Release events that rdev reports without them).
                let mut last_x = 0.0f64;
                let mut last_y = 0.0f64;

                if let Err(e) = listen(move |ev| {
                    if flag.load(Ordering::Relaxed) {
                        return;
                    }

                    let msg = match ev.event_type {
                        EventType::MouseMove { x, y } => {
                            last_x = x;
                            last_y = y;
                            Some(MouseEvent::Move { x, y })
                        }
                        EventType::ButtonPress(_) => Some(MouseEvent::ButtonPress {
                            x: last_x,
                            y: last_y,
                        }),
                        EventType::ButtonRelease(_) => Some(MouseEvent::ButtonRelease {
                            x: last_x,
                            y: last_y,
                        }),
                        _ => None,
                    };

                    if let Some(event) = msg {
                        if tx_inner.send(event).is_err() {
                            debug!("Mouse event receiver dropped");
                        }
                    }
                }) {
                    error!("rdev listen exited with error: {e:?}");
                }
            })
            .map_err(|e| ZoomError::TrackerInit(format!("Failed to spawn mouse tracker thread: {e}")))?;

        Ok((
            Self {
                stop_flag,
                _listener_thread: handle,
            },
            rx,
        ))
    }

    /// Signal the tracker to stop forwarding events.
    ///
    /// The underlying OS thread will exit on the next mouse event.
    pub fn stop(&mut self) {
        info!("Stopping mouse tracker");
        self.stop_flag.store(true, Ordering::SeqCst);
    }
}
