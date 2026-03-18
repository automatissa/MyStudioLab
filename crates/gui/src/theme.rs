/// MyStudioLab design tokens.
use egui::{Color32, CornerRadius, FontId, Stroke, Style, Visuals};

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

pub const BG_PRIMARY: Color32   = Color32::from_rgb(0x0d, 0x0d, 0x0d);
pub const BG_PANEL: Color32     = Color32::from_rgb(0x16, 0x16, 0x1a);
pub const BG_WIDGET: Color32    = Color32::from_rgb(0x1e, 0x1e, 0x26);
pub const BG_HOVER: Color32     = Color32::from_rgb(0x26, 0x26, 0x32);
pub const BG_ACTIVE: Color32    = Color32::from_rgb(0x2a, 0x2a, 0x3a);

pub const ACCENT: Color32       = Color32::from_rgb(0x00, 0xf6, 0xff);
pub const ACCENT_DIM: Color32   = Color32::from_rgb(0x00, 0xb8, 0xbf);

pub const TEXT_PRIMARY: Color32   = Color32::from_rgb(0xf0, 0xf0, 0xf5);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x88, 0x88, 0x99);

pub const DANGER: Color32  = Color32::from_rgb(0xff, 0x45, 0x60);
pub const SUCCESS: Color32 = Color32::from_rgb(0x39, 0xff, 0x88);

// egui 0.31: CornerRadius::same() takes u8
pub const ROUNDING: u8    = 8;
pub const ROUNDING_SM: u8 = 4;

// ---------------------------------------------------------------------------

pub fn apply(ctx: &egui::Context) {
    let mut style = Style::default();

    style.spacing.item_spacing    = egui::vec2(8.0, 6.0);
    style.spacing.button_padding  = egui::vec2(14.0, 8.0);
    style.spacing.slider_width    = 180.0;
    style.spacing.combo_width     = 180.0;
    style.spacing.icon_width      = 18.0;

    let mut v = Visuals::dark();

    v.override_text_color = Some(TEXT_PRIMARY);
    v.window_fill         = BG_PANEL;
    v.panel_fill          = BG_PRIMARY;
    v.window_stroke       = Stroke::new(1.0, ACCENT.linear_multiply(0.3));
    v.window_corner_radius = CornerRadius::same(ROUNDING);

    // noninteractive
    v.widgets.noninteractive.bg_fill      = BG_PANEL;
    v.widgets.noninteractive.bg_stroke    = Stroke::new(1.0, BG_WIDGET);
    v.widgets.noninteractive.fg_stroke    = Stroke::new(1.0, TEXT_SECONDARY);
    v.widgets.noninteractive.corner_radius = CornerRadius::same(ROUNDING_SM);

    // inactive
    v.widgets.inactive.bg_fill      = BG_WIDGET;
    v.widgets.inactive.bg_stroke    = Stroke::NONE;
    v.widgets.inactive.fg_stroke    = Stroke::new(1.5, TEXT_PRIMARY);
    v.widgets.inactive.corner_radius = CornerRadius::same(ROUNDING_SM);

    // hovered
    v.widgets.hovered.bg_fill      = BG_HOVER;
    v.widgets.hovered.bg_stroke    = Stroke::new(1.0, ACCENT.linear_multiply(0.6));
    v.widgets.hovered.fg_stroke    = Stroke::new(1.5, ACCENT);
    v.widgets.hovered.corner_radius = CornerRadius::same(ROUNDING_SM);
    v.widgets.hovered.expansion    = 1.0;

    // active
    v.widgets.active.bg_fill      = BG_ACTIVE;
    v.widgets.active.bg_stroke    = Stroke::new(1.5, ACCENT);
    v.widgets.active.fg_stroke    = Stroke::new(2.0, ACCENT);
    v.widgets.active.corner_radius = CornerRadius::same(ROUNDING_SM);

    // open
    v.widgets.open.bg_fill      = BG_ACTIVE;
    v.widgets.open.bg_stroke    = Stroke::new(1.5, ACCENT);
    v.widgets.open.fg_stroke    = Stroke::new(1.5, ACCENT);
    v.widgets.open.corner_radius = CornerRadius::same(ROUNDING_SM);

    v.selection.bg_fill = ACCENT.linear_multiply(0.25);
    v.selection.stroke  = Stroke::new(1.0, ACCENT);

    v.extreme_bg_color  = BG_PRIMARY;
    v.faint_bg_color    = BG_PANEL;
    v.code_bg_color     = BG_WIDGET;
    v.hyperlink_color   = ACCENT;

    style.visuals = v;

    style.text_styles.insert(egui::TextStyle::Body,     FontId::proportional(14.0));
    style.text_styles.insert(egui::TextStyle::Button,   FontId::proportional(14.0));
    style.text_styles.insert(egui::TextStyle::Heading,  FontId::proportional(20.0));
    style.text_styles.insert(egui::TextStyle::Small,    FontId::proportional(11.0));
    style.text_styles.insert(egui::TextStyle::Monospace, FontId::monospace(13.0));

    ctx.set_style(style);
}
