use egui::{Align, Layout, RichText, Ui};

use crate::{
    state::AppState,
    theme::{ACCENT, TEXT_SECONDARY},
    widgets::{card, labelled_row, section_header, status_bar},
};

pub fn draw(ui: &mut Ui, state: &mut AppState) {
    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.add_space(20.0);
        ui.label(RichText::new("Settings").size(22.0).color(ACCENT).strong());
    });
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);
        ui.horizontal_top(|ui| {
            ui.add_space(16.0);
            ui.vertical(|ui| {
                ui.set_max_width(520.0);

                // ---- Capture -----------------------------------------
                card(ui, |ui| {
                    section_header(ui, "Capture");

                    labelled_row(ui, "Display", |ui| {
                        egui::ComboBox::from_id_salt("settings_display")
                            .selected_text(format!("Display {}", state.settings.display_index))
                            .show_ui(ui, |ui| {
                                for i in 0..4 {
                                    ui.selectable_value(
                                        &mut state.settings.display_index,
                                        i,
                                        format!("Display {i}"),
                                    );
                                }
                            });
                    });

                    labelled_row(ui, "Frame rate", |ui| {
                        egui::ComboBox::from_id_salt("settings_fps")
                            .selected_text(format!("{} fps", state.settings.fps))
                            .show_ui(ui, |ui| {
                                for fps in [24u32, 30, 60] {
                                    ui.selectable_value(
                                        &mut state.settings.fps,
                                        fps,
                                        format!("{fps} fps"),
                                    );
                                }
                            });
                    });

                    labelled_row(ui, "Output file", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut state.settings.output_path)
                                .desired_width(260.0)
                                .hint_text("recording.mp4"),
                        );
                    });
                });

                ui.add_space(10.0);

                // ---- Auto-zoom ---------------------------------------
                card(ui, |ui| {
                    section_header(ui, "Auto-Zoom");

                    labelled_row(ui, "Enable", |ui| {
                        ui.checkbox(&mut state.settings.zoom_enabled, "");
                    });

                    ui.add_enabled_ui(state.settings.zoom_enabled, |ui| {
                        labelled_row(ui, "Max zoom", |ui| {
                            ui.add(
                                egui::Slider::new(&mut state.settings.max_zoom, 1.2..=4.0)
                                    .step_by(0.1)
                                    .suffix("×"),
                            );
                        });

                        labelled_row(ui, "Zoom-in speed", |ui| {
                            ui.add(
                                egui::Slider::new(&mut state.settings.zoom_in_secs, 0.1..=1.0)
                                    .step_by(0.05)
                                    .suffix(" s"),
                            );
                        });

                        labelled_row(ui, "Hold duration", |ui| {
                            ui.add(
                                egui::Slider::new(&mut state.settings.hold_secs, 0.3..=5.0)
                                    .step_by(0.1)
                                    .suffix(" s"),
                            );
                        });

                        labelled_row(ui, "Zoom-out speed", |ui| {
                            ui.add(
                                egui::Slider::new(&mut state.settings.zoom_out_secs, 0.1..=1.5)
                                    .step_by(0.05)
                                    .suffix(" s"),
                            );
                        });
                    });
                });

                ui.add_space(10.0);

                // ---- Audio -------------------------------------------
                card(ui, |ui| {
                    section_header(ui, "Audio");

                    labelled_row(ui, "Microphone", |ui| {
                        ui.checkbox(&mut state.settings.capture_mic, "Capture mic input");
                    });

                    labelled_row(ui, "System audio", |ui| {
                        ui.checkbox(
                            &mut state.settings.capture_system_audio,
                            "Capture loopback (not yet implemented)",
                        );
                    });

                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Audio muxing into the output file is coming in v0.2.")
                            .size(11.0)
                            .color(TEXT_SECONDARY),
                    );
                });

                ui.add_space(10.0);

                // ---- About -------------------------------------------
                card(ui, |ui| {
                    section_header(ui, "About");
                    ui.label(
                        RichText::new(concat!(
                            "MyStudioLab v",
                            env!("CARGO_PKG_VERSION"),
                        ))
                        .size(13.0),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(
                            "Free · offline · open-source · MIT license\n\
                             Powered by egui, windows-capture, rdev, and ffmpeg.",
                        )
                        .size(11.0)
                        .color(TEXT_SECONDARY),
                    );
                });
            });
        });
    });

    // Status bar
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        status_bar(ui, &state.status.clone(), state.last_error.is_some());
    });
}
