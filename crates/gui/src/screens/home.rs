use egui::{Align, Color32, CornerRadius, Layout, RichText, Stroke, Ui, Vec2};

use crate::{
    state::AppState,
    theme::{ACCENT, BORDER, DANGER, SECONDARY_BG, SUCCESS, TEXT, TEXT_DIM},
};

pub fn draw(ui: &mut Ui, state: &mut AppState) {
    let is_recording = state.is_recording();

    // ── Outer padding ─────────────────────────────────────────────────────────
    egui::Frame::new()
        .inner_margin(egui::Margin::same(32))
        .fill(crate::theme::PRIMARY_BG)
        .show(ui, |ui| {
            ui.vertical(|ui| {

                // ── Top: title + status dot ───────────────────────────────────
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Record").size(22.0).color(TEXT).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if is_recording {
                            let t = ui.input(|i| i.time);
                            let alpha = ((t * 2.5).sin() * 0.5 + 0.5) as f32;
                            let dot = Color32::from_rgba_unmultiplied(
                                DANGER.r(), DANGER.g(), DANGER.b(),
                                (220.0 * alpha) as u8,
                            );
                            let secs = state.elapsed.as_secs();
                            ui.label(
                                RichText::new(format!(
                                    "{:02}:{:02}:{:02}",
                                    secs / 3600, (secs % 3600) / 60, secs % 60
                                ))
                                .size(13.0).color(DANGER).strong(),
                            );
                            ui.add_space(4.0);
                            let (resp, painter) = ui.allocate_painter(
                                Vec2::splat(10.0), egui::Sense::hover(),
                            );
                            painter.circle_filled(resp.rect.center(), 4.5, dot);
                        } else {
                            let (resp, painter) = ui.allocate_painter(
                                Vec2::splat(10.0), egui::Sense::hover(),
                            );
                            painter.circle_filled(resp.rect.center(), 4.5, SUCCESS);
                            ui.add_space(4.0);
                            ui.label(RichText::new("Ready").size(12.0).color(SUCCESS));
                        }
                    });
                });

                ui.add_space(32.0);

                // ── Big record / stop button ──────────────────────────────────
                ui.vertical_centered(|ui| {
                    let btn_size = Vec2::new(200.0, 200.0);

                    if is_recording {
                        // Stop — red filled circle with square icon
                        if record_circle_button(ui, btn_size, DANGER, "⏹", "Stop Recording") {
                            state.stop_recording();
                        }
                    } else {
                        // Record — accent filled circle
                        if record_circle_button(ui, btn_size, ACCENT, "⏺", "Start Recording") {
                            if let Err(e) = state.start_recording() {
                                state.last_error = Some(e.clone());
                                state.status     = e;
                            }
                        }
                    }
                });

                ui.add_space(32.0);
                ui.separator();
                ui.add_space(20.0);

                // ── Quick settings strip (hidden during recording) ────────────
                if !is_recording {
                    quick_settings(ui, state);
                    ui.add_space(20.0);
                }

                // ── Output path ───────────────────────────────────────────────
                output_row(ui, state, is_recording);

                // ── Error banner ──────────────────────────────────────────────
                if let Some(err) = state.last_error.clone() {
                    ui.add_space(16.0);
                    egui::Frame::new()
                        .fill(Color32::from_rgba_unmultiplied(
                            DANGER.r(), DANGER.g(), DANGER.b(), 30,
                        ))
                        .stroke(Stroke::new(1.0, DANGER.linear_multiply(0.5)))
                        .corner_radius(CornerRadius::same(crate::theme::RADIUS_SM))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("⚠  ").color(DANGER));
                                ui.label(RichText::new(&err).color(DANGER).size(12.0));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.small_button("✕").clicked() {
                                        state.last_error = None;
                                    }
                                });
                            });
                        });
                }

                // ── Status bar at very bottom ─────────────────────────────────
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.label(
                        RichText::new(&state.status)
                            .size(11.0)
                            .color(TEXT_DIM),
                    );
                });
            });
        });
}

// ── Big circle button ─────────────────────────────────────────────────────────

fn record_circle_button(
    ui: &mut Ui,
    size: Vec2,
    color: Color32,
    icon: &str,
    sublabel: &str,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center  = rect.center();
        let radius  = size.x / 2.0;

        // Shadow ring
        painter.circle_stroke(center, radius + 1.0, Stroke::new(6.0, color.linear_multiply(0.12)));

        // Fill
        let fill = if response.hovered() {
            color.linear_multiply(0.88)
        } else {
            color
        };
        painter.circle_filled(center, radius, fill);

        // Icon
        painter.text(
            center - Vec2::new(0.0, 8.0),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(42.0),
            if color == crate::theme::ACCENT { Color32::BLACK } else { Color32::WHITE },
        );

        // Sub-label below icon
        painter.text(
            center + Vec2::new(0.0, 22.0),
            egui::Align2::CENTER_CENTER,
            sublabel,
            egui::FontId::proportional(12.0),
            if color == crate::theme::ACCENT {
                Color32::from_rgba_unmultiplied(0, 0, 0, 180)
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 180)
            },
        );
    }

    response.clicked()
}

// ── Quick settings strip ──────────────────────────────────────────────────────

fn quick_settings(ui: &mut Ui, state: &mut AppState) {
    // Three pill-shaped toggles in a row: FPS | Zoom | Mic
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;

        // FPS selector
        let fps_label = format!("{}fps", state.settings.fps);
        pill_combo(ui, "FPS", |ui| {
            for fps in [24u32, 30, 60] {
                ui.selectable_value(
                    &mut state.settings.fps,
                    fps,
                    format!("{fps}"),
                );
            }
        }, &fps_label);

        // Zoom toggle + level
        let zoom_label = if state.settings.zoom_enabled {
            format!("Zoom  {:.1}×", state.settings.max_zoom)
        } else {
            "Zoom  Off".into()
        };
        pill_toggle(ui, &zoom_label, &mut state.settings.zoom_enabled);

        if state.settings.zoom_enabled {
            ui.add(
                egui::Slider::new(&mut state.settings.max_zoom, 1.2..=4.0)
                    .step_by(0.1)
                    .suffix("×")
                    .show_value(false),
            );
        }

        // Mic toggle
        let mic_label = if state.settings.capture_mic { "Mic  On" } else { "Mic  Off" };
        pill_toggle(ui, mic_label, &mut state.settings.capture_mic);
    });
}

fn pill_combo(ui: &mut Ui, _label: &str, add_items: impl FnOnce(&mut Ui), selected_text: &str) {
    egui::Frame::new()
        .fill(SECONDARY_BG)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(egui::Margin { left: 14, right: 8, top: 6, bottom: 6 })
        .show(ui, |ui| {
            egui::ComboBox::from_id_salt(selected_text)
                .selected_text(RichText::new(selected_text).size(13.0).color(TEXT))
                .show_ui(ui, add_items);
        });
}

fn pill_toggle(ui: &mut Ui, label: &str, value: &mut bool) {
    let (fill, text_color) = if *value {
        (Color32::from_rgb(0x00, 0x45, 0x45), ACCENT)
    } else {
        (SECONDARY_BG, TEXT_DIM)
    };

    let frame = egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, if *value { ACCENT.linear_multiply(0.4) } else { BORDER }))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(egui::Margin { left: 14, right: 14, top: 7, bottom: 7 });

    if frame.show(ui, |ui| {
        ui.label(RichText::new(label).size(13.0).color(text_color))
    }).response.clicked() {
        *value = !*value;
    }
}

// ── Output path row ───────────────────────────────────────────────────────────

fn output_row(ui: &mut Ui, state: &mut AppState, is_recording: bool) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("📁").size(14.0));
        ui.add_space(4.0);
        if is_recording {
            ui.label(
                RichText::new(&state.settings.output_path)
                    .size(12.0)
                    .color(TEXT_DIM),
            );
        } else {
            ui.add(
                egui::TextEdit::singleline(&mut state.settings.output_path)
                    .desired_width(ui.available_width() - 8.0)
                    .font(egui::FontId::proportional(12.0))
                    .hint_text("Output path…"),
            );
        }
    });
}
