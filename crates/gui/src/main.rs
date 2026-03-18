// On Windows: don't pop a console window behind the GUI.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod screens;
mod state;
mod theme;
#[allow(dead_code)]
mod widgets;

use anyhow::Result;
use eframe::NativeOptions;
use egui::ViewportBuilder;

fn main() -> Result<()> {
    // Logging — shows in a terminal if launched from one, silent otherwise.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mystudiolab_gui=info,capture=info,zoom=info,encode=info".into()),
        )
        .init();

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("MyStudioLab")
            .with_inner_size([640.0, 480.0])
            .with_min_inner_size([520.0, 420.0])
            .with_resizable(true)
            .with_decorations(true),
        ..Default::default()
    };

    eframe::run_native(
        "MyStudioLab",
        options,
        Box::new(|cc| Ok(Box::new(app::MyStudioLabApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}
