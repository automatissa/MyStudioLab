use egui::{RichText, Ui};

use crate::{
    state::AppState,
    theme::{ACCENT, TEXT, TEXT_DIM},
};

pub fn draw(ui: &mut Ui, state: &mut AppState) {
    egui::Frame::new()
        .inner_margin(egui::Margin::same(32))
        .fill(crate::theme::PRIMARY_BG)
        .show(ui, |ui| {
            ui.label(RichText::new("Settings").size(22.0).color(TEXT).strong());
            ui.add_space(24.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_max_width(480.0);

                // ── Capture ───────────────────────────────────────────────────
                section(ui, "Capture");

                row(ui, "Display", |ui| {
                    egui::ComboBox::from_id_salt("set_display")
                        .selected_text(format!("Display {}", state.settings.display_index))
                        .show_ui(ui, |ui| {
                            for i in 0..4 {
                                ui.selectable_value(
                                    &mut state.settings.display_index, i,
                                    format!("Display {i}"),
                                );
                            }
                        });
                });

                row(ui, "Frame rate", |ui| {
                    egui::ComboBox::from_id_salt("set_fps")
                        .selected_text(format!("{} fps", state.settings.fps))
                        .show_ui(ui, |ui| {
                            for fps in [24u32, 30, 60] {
                                ui.selectable_value(
                                    &mut state.settings.fps, fps,
                                    format!("{fps} fps"),
                                );
                            }
                        });
                });

                row(ui, "Output file", |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut state.settings.output_path)
                            .desired_width(280.0)
                            .hint_text("recording.mp4"),
                    );
                });

                ui.add_space(20.0);

                // ── Auto-Zoom ─────────────────────────────────────────────────
                section(ui, "Auto-Zoom");

                row(ui, "Enable", |ui| {
                    ui.checkbox(&mut state.settings.zoom_enabled, "");
                });

                ui.add_enabled_ui(state.settings.zoom_enabled, |ui| {
                    row(ui, "Zoom level", |ui| {
                        ui.add(
                            egui::Slider::new(&mut state.settings.max_zoom, 1.2..=4.0)
                                .step_by(0.1)
                                .suffix("×"),
                        );
                    });
                    row(ui, "Zoom-in", |ui| {
                        ui.add(
                            egui::Slider::new(&mut state.settings.zoom_in_secs, 0.1..=1.0)
                                .step_by(0.05)
                                .suffix(" s"),
                        );
                    });
                    row(ui, "Hold", |ui| {
                        ui.add(
                            egui::Slider::new(&mut state.settings.hold_secs, 0.3..=5.0)
                                .step_by(0.1)
                                .suffix(" s"),
                        );
                    });
                    row(ui, "Zoom-out", |ui| {
                        ui.add(
                            egui::Slider::new(&mut state.settings.zoom_out_secs, 0.1..=1.5)
                                .step_by(0.05)
                                .suffix(" s"),
                        );
                    });
                });

                ui.add_space(20.0);

                // ── Audio ─────────────────────────────────────────────────────
                section(ui, "Audio");

                row(ui, "Microphone", |ui| {
                    ui.checkbox(&mut state.settings.capture_mic, "Capture mic input");
                });
                row(ui, "System audio", |ui| {
                    ui.add_enabled(
                        false,
                        egui::Checkbox::new(
                            &mut state.settings.capture_system_audio,
                            "Loopback  (coming in v1.1)",
                        ),
                    );
                });

                ui.add_space(20.0);

                // ── About ─────────────────────────────────────────────────────
                section(ui, "About");
                ui.add_space(4.0);
                ui.label(
                    RichText::new(concat!("MyStudioLab  v", env!("CARGO_PKG_VERSION")))
                        .size(13.0)
                        .color(TEXT),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new("Free · offline · open-source · MIT\nPowered by egui, windows-capture, rdev and ffmpeg.")
                        .size(11.0)
                        .color(TEXT_DIM),
                );
            });
        });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn section(ui: &mut Ui, label: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(3.0, 14.0), egui::Sense::hover(),
        );
        ui.painter().rect_filled(rect, egui::CornerRadius::same(2), ACCENT);
        ui.add_space(8.0);
        ui.label(RichText::new(label).size(12.0).color(TEXT_DIM).strong());
    });
    ui.add_space(8.0);
}

fn row(ui: &mut Ui, label: &str, add_widget: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [150.0, 20.0],
            egui::Label::new(RichText::new(label).size(13.0).color(TEXT_DIM)),
        );
        add_widget(ui);
    });
    ui.add_space(4.0);
}
