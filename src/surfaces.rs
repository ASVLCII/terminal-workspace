//! Shared matte shell styling. Terminal rendering and docking geometry stay independent.
use eframe::egui::{self, Color32};

pub const CONTROL_RADIUS: egui::CornerRadius = egui::CornerRadius::same(6);
pub const FIELD_RADIUS: egui::CornerRadius = egui::CornerRadius::same(8);
pub const FLOATING_RADIUS: egui::CornerRadius = egui::CornerRadius::same(12);
pub const CONTROL: Color32 = Color32::from_rgb(25, 25, 25);
pub const CONTROL_EDGE: Color32 = Color32::from_rgb(48, 48, 48);
pub const OVERLAY_SHADOW: egui::epaint::Shadow = egui::epaint::Shadow {
    offset: [0, 4],
    blur: 12,
    spread: 0,
    color: Color32::from_black_alpha(72),
};

pub const MUTED: Color32 = Color32::from_rgb(160, 160, 160);
pub const LINE: Color32 = Color32::from_rgb(41, 41, 41);
pub const CHROME: Color32 = Color32::from_rgb(17, 17, 17);
pub const SIDEBAR: Color32 = Color32::from_rgb(16, 16, 16);
pub const CANVAS: Color32 = Color32::from_rgb(8, 8, 8);
pub const SELECTED: Color32 = Color32::from_rgb(36, 36, 36);
pub const HOVER: Color32 = Color32::from_rgb(30, 30, 30);
pub const FG: Color32 = Color32::from_rgb(240, 240, 240);

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = SIDEBAR;
    style.visuals.window_fill = CHROME;
    style.visuals.extreme_bg_color = CANVAS;
    style.visuals.override_text_color = Some(FG);
    style.visuals.selection.bg_fill = SELECTED;
    style.visuals.selection.stroke = egui::Stroke::new(1., Color32::from_gray(180));
    style.visuals.hyperlink_color = Color32::from_gray(210);
    style.visuals.window_stroke = egui::Stroke::new(1., LINE);
    style.visuals.window_corner_radius = FLOATING_RADIUS;
    style.visuals.window_shadow = OVERLAY_SHADOW;
    style.visuals.menu_corner_radius = FLOATING_RADIUS;
    style.visuals.popup_shadow = OVERLAY_SHADOW;
    style.visuals.widgets.inactive.bg_fill = CONTROL;
    style.visuals.widgets.inactive.weak_bg_fill = CONTROL;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1., CONTROL_EDGE);
    style.visuals.widgets.hovered.bg_fill = HOVER;
    style.visuals.widgets.hovered.weak_bg_fill = HOVER;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1., Color32::from_gray(105));
    style.visuals.widgets.active.bg_fill = SELECTED;
    style.visuals.widgets.active.weak_bg_fill = SELECTED;
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1., Color32::from_gray(180));
    style.visuals.widgets.open.bg_fill = SELECTED;
    style.visuals.widgets.open.weak_bg_fill = SELECTED;
    for visual in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
        &mut style.visuals.widgets.noninteractive,
    ] {
        visual.corner_radius = CONTROL_RADIUS;
    }
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(14.));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(14.));
    style
        .text_styles
        .insert(egui::TextStyle::Small, egui::FontId::proportional(12.));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(20.));
    style.spacing.interact_size = egui::vec2(32., 32.);
    style.spacing.item_spacing = egui::vec2(8., 8.);
    style.spacing.button_padding = egui::vec2(12., 7.);
    style.spacing.window_margin = egui::Margin::same(20);
    ctx.set_style_of(egui::Theme::Dark, style);
}
