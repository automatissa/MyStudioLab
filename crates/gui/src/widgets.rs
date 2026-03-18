/// Reusable UI widgets and helpers.
use egui::{Color32, CornerRadius, Rect, Response, Stroke, Ui, Vec2};

use crate::theme::{ACCENT, BORDER, DANGER, RADIUS, SECONDARY_BG, TEXT, TEXT_DIM};

// ---------------------------------------------------------------------------
// Section header
// ---------------------------------------------------------------------------

pub fn section_header(ui: &mut Ui, label: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        // allocate_exact_size returns (Rect, Response) in egui 0.31
        let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(3.0, 16.0), egui::Sense::hover());
        ui.painter().rect_filled(bar_rect, CornerRadius::same(2), ACCENT);
        ui.add_space(6.0);
        ui.label(egui::RichText::new(label).size(13.0).color(TEXT).strong());
    });
    ui.add_space(4.0);
}

// ---------------------------------------------------------------------------
// Accent / danger buttons
// ---------------------------------------------------------------------------

pub fn accent_button(ui: &mut Ui, label: &str) -> Response {
    let button = egui::Button::new(
        egui::RichText::new(label).color(Color32::BLACK).strong().size(14.0),
    )
    .fill(ACCENT)
    .corner_radius(CornerRadius::same(RADIUS))
    .min_size(Vec2::new(140.0, 40.0));

    ui.add(button)
}

pub fn danger_button(ui: &mut Ui, label: &str) -> Response {
    let button = egui::Button::new(
        egui::RichText::new(label).color(Color32::WHITE).strong().size(14.0),
    )
    .fill(DANGER)
    .corner_radius(CornerRadius::same(RADIUS))
    .min_size(Vec2::new(140.0, 40.0));

    ui.add(button)
}

// ---------------------------------------------------------------------------
// Labelled row
// ---------------------------------------------------------------------------

pub fn labelled_row(ui: &mut Ui, label: &str, add_widget: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [140.0, 20.0],
            egui::Label::new(egui::RichText::new(label).color(TEXT_DIM).size(13.0)),
        );
        add_widget(ui);
    });
}

// ---------------------------------------------------------------------------
// Recording indicator
// ---------------------------------------------------------------------------

pub fn rec_indicator(ui: &mut Ui, elapsed: std::time::Duration) {
    let secs = elapsed.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;

    ui.horizontal(|ui| {
        let t = ui.input(|i| i.time);
        let alpha = ((t * 2.5).sin() * 0.5 + 0.5) as f32;
        let dot_color = Color32::from_rgba_unmultiplied(
            DANGER.r(), DANGER.g(), DANGER.b(),
            (200.0 * alpha) as u8,
        );

        let (resp, painter) = ui.allocate_painter(Vec2::splat(12.0), egui::Sense::hover());
        painter.circle_filled(resp.rect.center(), 5.0, dot_color);

        ui.label(
            egui::RichText::new(format!("REC  {:02}:{:02}:{:02}", h, m, s))
                .color(DANGER).strong().size(14.0),
        );
    });
}

// ---------------------------------------------------------------------------
// Zoom preview bar
// ---------------------------------------------------------------------------

pub fn zoom_preview_bar(ui: &mut Ui, zoom: f64, max_zoom: f64) {
    let fraction = ((zoom - 1.0) / (max_zoom - 1.0).max(0.001)).clamp(0.0, 1.0) as f32;
    let desired = Vec2::new(ui.available_width().min(200.0), 6.0);
    let (r, painter) = ui.allocate_painter(desired, egui::Sense::hover());
    let r = r.rect;

    painter.rect_filled(r, CornerRadius::same(3), SECONDARY_BG);
    if fraction > 0.0 {
        let filled = Rect::from_min_size(r.min, Vec2::new(r.width() * fraction, r.height()));
        painter.rect_filled(filled, CornerRadius::same(3), ACCENT);
    }
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

pub fn status_bar(ui: &mut Ui, msg: &str, is_error: bool) {
    let color = if is_error { DANGER } else { TEXT_DIM };
    ui.separator();
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(msg).color(color).size(11.0));
    });
}

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

pub fn card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(crate::theme::SECONDARY_BG)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(egui::Margin::same(14))
        .show(ui, add_contents);
}
