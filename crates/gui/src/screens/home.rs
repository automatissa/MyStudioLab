use egui::{Align, Layout, RichText, Ui};

use crate::{
    state::AppState,
    theme::{ACCENT, DANGER, SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY},
    widgets::{accent_button, card, danger_button, labelled_row, rec_indicator, section_header, status_bar, zoom_preview_bar},
};

pub fn draw(ui: &mut Ui, state: &mut AppState) {
    // ---- Header -------------------------------------------------------
    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.add_space(20.0);
        ui.vertical(|ui| {
            ui.label(
                RichText::new("MyStudioLab")
                    .size(26.0)
                    .color(ACCENT)
                    .strong(),
            );
            ui.label(
                RichText::new("Free · Offline · Open-source screen recorder")
                    .size(12.0)
                    .color(TEXT_SECONDARY),
            );
        });
    });
    ui.add_space(16.0);
    ui.separator();
    ui.add_space(12.0);

    let is_recording = state.is_recording();

    // ---- Main content split —left panel + right panel ----------------
    ui.horizontal_top(|ui| {
        ui.add_space(16.0);

        // ---- LEFT: controls -----------------------------------------
        ui.vertical(|ui| {
            ui.set_min_width(260.0);

            // --- Recording indicator / big button --------------------
            card(ui, |ui| {
                if is_recording {
                    rec_indicator(ui, state.elapsed);
                    ui.add_space(14.0);
                    zoom_preview_bar(ui, 1.0, state.settings.max_zoom);
                    ui.add_space(14.0);
                    if danger_button(ui, "  ⏹  Stop Recording").clicked() {
                        state.stop_recording();
                    }
                } else {
                    ui.label(
                        RichText::new("Ready to record")
                            .color(SUCCESS)
                            .size(13.0),
                    );
                    ui.add_space(10.0);
                    if accent_button(ui, "  ⏺  Start Recording").clicked() {
                        if let Err(e) = state.start_recording() {
                            state.last_error = Some(e.clone());
                            state.status = e;
                        }
                    }
                }
            });

            ui.add_space(12.0);

            // --- Quick settings (non-recording only) ------------------
            if !is_recording {
                card(ui, |ui| {
                    section_header(ui, "Quick Settings");

                    labelled_row(ui, "Output file", |ui| {
                        let te = egui::TextEdit::singleline(&mut state.settings.output_path)
                            .desired_width(180.0);
                        ui.add(te);
                    });

                    labelled_row(ui, "Display", |ui| {
                        egui::ComboBox::from_id_salt("display_combo")
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

                    labelled_row(ui, "FPS", |ui| {
                        egui::ComboBox::from_id_salt("fps_combo")
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

                    labelled_row(ui, "Auto-zoom", |ui| {
                        ui.checkbox(&mut state.settings.zoom_enabled, "");
                    });

                    if state.settings.zoom_enabled {
                        labelled_row(ui, "Zoom level", |ui| {
                            ui.add(
                                egui::Slider::new(&mut state.settings.max_zoom, 1.2..=4.0)
                                    .step_by(0.1)
                                    .suffix("×"),
                            );
                        });
                    }
                });
            }
        }); // end LEFT

        ui.add_space(12.0);

        // ---- RIGHT: info / stats ------------------------------------
        ui.vertical(|ui| {
            ui.set_min_width(200.0);

            card(ui, |ui| {
                section_header(ui, "Session Info");

                info_row(ui, "Output", &state.settings.output_path);
                info_row(ui, "Display",
                    &format!("Display {}", state.settings.display_index));
                info_row(ui, "FPS", &state.settings.fps.to_string());
                info_row(ui, "Auto-zoom",
                    if state.settings.zoom_enabled { "On" } else { "Off" });

                if state.settings.zoom_enabled {
                    info_row(ui, "Max zoom",
                        &format!("{:.1}×", state.settings.max_zoom));
                }

                if is_recording {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);
                    let secs = state.elapsed.as_secs();
                    info_row(
                        ui,
                        "Elapsed",
                        &format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60),
                    );
                }
            });

            ui.add_space(12.0);

            // Error banner
            if let Some(err) = &state.last_error.clone() {
                card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚠").color(DANGER).size(16.0));
                        ui.add_space(4.0);
                        ui.label(RichText::new(err).color(DANGER).size(12.0));
                    });
                    ui.add_space(4.0);
                    if ui.small_button("Dismiss").clicked() {
                        state.last_error = None;
                    }
                });
            }
        }); // end RIGHT
    }); // end horizontal

    // ---- Status bar --------------------------------------------------
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        status_bar(ui, &state.status.clone(), state.last_error.is_some());
    });
}

// ---------------------------------------------------------------------------
// Tiny helper
// ---------------------------------------------------------------------------

fn info_row(ui: &mut Ui, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).color(TEXT_SECONDARY).size(12.0));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).color(TEXT_PRIMARY).size(12.0));
        });
    });
}
