use eframe::CreationContext;
use egui::{Color32, CornerRadius, RichText, Stroke, Ui, Vec2};

use crate::{
    screens,
    state::{AppScreen, AppState},
    theme::{self, ACCENT, BORDER, NAV_ACTIVE_BG, PRIMARY_BG, SECONDARY_BG, TEXT_DIM},
};

// ── App ───────────────────────────────────────────────────────────────────────

pub struct MyStudioLabApp {
    state:         AppState,
    theme_applied: bool,
}

impl MyStudioLabApp {
    pub fn new(_cc: &CreationContext) -> Self {
        Self { state: AppState::default(), theme_applied: false }
    }
}

impl eframe::App for MyStudioLabApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            theme::apply(ctx);
            self.theme_applied = true;
        }

        if self.state.is_recording() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }

        self.state.tick();

        // ── Sidebar ───────────────────────────────────────────────────────────
        egui::SidePanel::left("sidebar")
            .exact_width(220.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(SECONDARY_BG)
                    .inner_margin(egui::Margin::ZERO)
                    .stroke(Stroke::new(1.0, BORDER)),
            )
            .show(ctx, |ui| {
                draw_sidebar(ui, &mut self.state);
            });

        // ── Main content ──────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(PRIMARY_BG))
            .show(ctx, |ui| {
                match self.state.screen {
                    AppScreen::Record   => screens::home::draw(ui, &mut self.state),
                    AppScreen::Settings => screens::settings::draw(ui, &mut self.state),
                }
            });
    }
}

// ── Sidebar ───────────────────────────────────────────────────────────────────

fn draw_sidebar(ui: &mut Ui, state: &mut AppState) {
    ui.set_min_height(ui.available_height());

    // Brand
    egui::Frame::new()
        .inner_margin(egui::Margin { left: 20, right: 20, top: 24, bottom: 16 })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("●").size(18.0).color(ACCENT));
                ui.add_space(6.0);
                ui.label(RichText::new("MyStudioLab").size(20.0).color(ACCENT).strong());
            });
        });

    ui.add_space(8.0);

    // Nav items
    egui::Frame::new()
        .inner_margin(egui::Margin { left: 12, right: 12, top: 0, bottom: 0 })
        .show(ui, |ui| {
            nav_item(ui, state, AppScreen::Record,   "⏺",  "Record");
            ui.add_space(4.0);
            nav_item(ui, state, AppScreen::Settings, "⚙",  "Settings");
        });

    // Version at bottom
    let available = ui.available_height();
    if available > 0.0 {
        ui.add_space(available - 28.0);
    }
    egui::Frame::new()
        .inner_margin(egui::Margin { left: 20, right: 20, top: 0, bottom: 12 })
        .show(ui, |ui| {
            ui.label(
                RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                    .size(11.0)
                    .color(TEXT_DIM),
            );
        });
}

fn nav_item(ui: &mut Ui, state: &mut AppState, screen: AppScreen, icon: &str, label: &str) {
    let active = state.screen == screen;

    let (bg, text_color) = if active {
        (NAV_ACTIVE_BG, ACCENT)
    } else {
        (Color32::TRANSPARENT, TEXT_DIM)
    };

    let desired = Vec2::new(ui.available_width(), 42.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, CornerRadius::same(6), bg);

        // Hover highlight
        if response.hovered() && !active {
            painter.rect_filled(
                rect,
                CornerRadius::same(6),
                Color32::from_rgb(0x33, 0x33, 0x33),
            );
        }

        // Left accent bar when active
        if active {
            let bar = egui::Rect::from_min_size(
                rect.min,
                Vec2::new(3.0, rect.height()),
            );
            painter.rect_filled(bar, CornerRadius::same(2), ACCENT);
        }

        // Icon + label
        let text_pos = rect.min + Vec2::new(18.0, rect.height() / 2.0);
        painter.text(
            text_pos,
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::proportional(15.0),
            if active { ACCENT } else { TEXT_DIM },
        );
        painter.text(
            text_pos + Vec2::new(26.0, 0.0),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(14.0),
            text_color,
        );
    }

    if response.clicked() {
        state.screen = screen;
    }
}
