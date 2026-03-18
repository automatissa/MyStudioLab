use eframe::CreationContext;
use egui::{Color32, CornerRadius, RichText, Stroke, Ui, Vec2};

use crate::{
    screens,
    state::{AppScreen, AppState},
    theme::{self, ACCENT, BG_PRIMARY, TEXT_PRIMARY, TEXT_SECONDARY},
};

// ---------------------------------------------------------------------------
// App struct
// ---------------------------------------------------------------------------

pub struct MyStudioLabApp {
    state: AppState,
    theme_applied: bool,
}

impl MyStudioLabApp {
    pub fn new(_cc: &CreationContext) -> Self {
        Self {
            state: AppState::default(),
            theme_applied: false,
        }
    }
}

// ---------------------------------------------------------------------------
// eframe::App implementation
// ---------------------------------------------------------------------------

impl eframe::App for MyStudioLabApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme once (egui persists styles across frames)
        if !self.theme_applied {
            theme::apply(ctx);
            self.theme_applied = true;
        }

        // Keep repainting while recording (drives the elapsed timer)
        if self.state.is_recording() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }

        self.state.tick();

        // Set window background to match our palette
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG_PRIMARY)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                draw_nav(ui, &mut self.state);

                match self.state.screen {
                    AppScreen::Home     => screens::home::draw(ui, &mut self.state),
                    AppScreen::Settings => screens::settings::draw(ui, &mut self.state),
                }
            });
    }
}

// ---------------------------------------------------------------------------
// Top navigation bar
// ---------------------------------------------------------------------------

fn draw_nav(ui: &mut Ui, state: &mut AppState) {
    egui::Frame::new()
        .fill(crate::theme::BG_PANEL)
        .inner_margin(egui::Margin { left: 16, right: 16, top: 0, bottom: 0 })
        .show(ui, |ui| {
            ui.set_min_height(44.0);
            ui.horizontal(|ui| {
                ui.add_space(0.0);

                // Logo mark
                ui.label(
                    RichText::new("●")
                        .size(18.0)
                        .color(ACCENT),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new("MyStudioLab")
                        .size(15.0)
                        .color(TEXT_PRIMARY)
                        .strong(),
                );

                ui.add_space(24.0);
                ui.separator();
                ui.add_space(16.0);

                // Nav tabs
                nav_tab(ui, state, AppScreen::Home,     "⏺  Record");
                ui.add_space(4.0);
                nav_tab(ui, state, AppScreen::Settings, "⚙  Settings");
            });
        });
}

fn nav_tab(ui: &mut Ui, state: &mut AppState, screen: AppScreen, label: &str) {
    let active = state.screen == screen;
    let color  = if active { ACCENT } else { TEXT_SECONDARY };
    let size   = Vec2::new(110.0, 44.0);

    let btn = egui::Button::new(RichText::new(label).color(color).size(13.0))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .min_size(size);

    let resp = ui.add(btn);

    // Underline accent bar for active tab
    if active {
        let bar_rect = {
            let r = resp.rect;
            egui::Rect::from_min_size(
                egui::Pos2::new(r.min.x, r.max.y - 2.0),
                Vec2::new(r.width(), 2.0),
            )
        };
        ui.painter().rect_filled(bar_rect, CornerRadius::ZERO, ACCENT);
    }

    if resp.clicked() {
        state.screen = screen;
    }
}
