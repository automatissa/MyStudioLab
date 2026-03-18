use egui::{Color32, CornerRadius, FontId, Stroke, Style, Visuals};

// ── Palette (matches MyFileLab Python app) ────────────────────────────────────
pub const PRIMARY_BG:   Color32 = Color32::from_rgb(0x0E, 0x0E, 0x0E);
pub const SECONDARY_BG: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);
pub const ACCENT:       Color32 = Color32::from_rgb(0x00, 0xf6, 0xff);
pub const TEXT:         Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
pub const TEXT_DIM:     Color32 = Color32::from_rgb(0x88, 0x88, 0x99);
pub const BORDER:       Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const DANGER:       Color32 = Color32::from_rgb(0xff, 0x45, 0x60);
pub const SUCCESS:      Color32 = Color32::from_rgb(0x30, 0xd1, 0x58); // system green

// Nav selected tint: #004545
pub const NAV_ACTIVE_BG: Color32 = Color32::from_rgb(0x00, 0x45, 0x45);

pub const RADIUS: u8    = 8;
pub const RADIUS_SM: u8 = 5;

// ── Apply ──────────────────────────────────────────────────────────────────────
pub fn apply(ctx: &egui::Context) {
    let mut style = Style::default();

    style.spacing.item_spacing   = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.slider_width   = 160.0;
    style.spacing.combo_width    = 160.0;
    style.spacing.icon_width     = 18.0;

    let mut v = Visuals::dark();
    v.override_text_color  = Some(TEXT);
    v.window_fill          = SECONDARY_BG;
    v.panel_fill           = PRIMARY_BG;
    v.window_stroke        = Stroke::new(1.0, BORDER);
    v.window_corner_radius = CornerRadius::same(RADIUS);
    v.extreme_bg_color     = PRIMARY_BG;
    v.faint_bg_color       = SECONDARY_BG;
    v.code_bg_color        = SECONDARY_BG;
    v.hyperlink_color      = ACCENT;

    // Noninteractive (labels, separators)
    v.widgets.noninteractive.bg_fill       = SECONDARY_BG;
    v.widgets.noninteractive.bg_stroke     = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.fg_stroke     = Stroke::new(1.0, TEXT_DIM);
    v.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_SM);

    // Idle
    v.widgets.inactive.bg_fill       = SECONDARY_BG;
    v.widgets.inactive.bg_stroke     = Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke     = Stroke::new(1.0, TEXT);
    v.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_SM);

    // Hovered
    v.widgets.hovered.bg_fill       = Color32::from_rgb(0x33, 0x33, 0x33);
    v.widgets.hovered.bg_stroke     = Stroke::new(1.0, ACCENT.linear_multiply(0.5));
    v.widgets.hovered.fg_stroke     = Stroke::new(1.5, ACCENT);
    v.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_SM);
    v.widgets.hovered.expansion     = 1.0;

    // Active / pressed
    v.widgets.active.bg_fill       = Color32::from_rgb(0x2a, 0x2a, 0x2a);
    v.widgets.active.bg_stroke     = Stroke::new(1.5, ACCENT);
    v.widgets.active.fg_stroke     = Stroke::new(2.0, ACCENT);
    v.widgets.active.corner_radius = CornerRadius::same(RADIUS_SM);

    // Open (combo box etc.)
    v.widgets.open.bg_fill       = Color32::from_rgb(0x2a, 0x2a, 0x2a);
    v.widgets.open.bg_stroke     = Stroke::new(1.5, ACCENT);
    v.widgets.open.fg_stroke     = Stroke::new(1.5, ACCENT);
    v.widgets.open.corner_radius = CornerRadius::same(RADIUS_SM);

    v.selection.bg_fill = ACCENT.linear_multiply(0.22);
    v.selection.stroke  = Stroke::new(1.0, ACCENT);

    style.visuals = v;

    style.text_styles.insert(egui::TextStyle::Body,      FontId::proportional(14.0));
    style.text_styles.insert(egui::TextStyle::Button,    FontId::proportional(14.0));
    style.text_styles.insert(egui::TextStyle::Heading,   FontId::proportional(20.0));
    style.text_styles.insert(egui::TextStyle::Small,     FontId::proportional(11.0));
    style.text_styles.insert(egui::TextStyle::Monospace, FontId::monospace(13.0));

    ctx.set_style(style);
}
