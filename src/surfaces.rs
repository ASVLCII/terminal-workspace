//! Shared glass surfaces and accent colors.
use eframe::egui::{self, Color32};
use terminal_workspace::model::Accent;

pub const CONTROL_RADIUS: egui::CornerRadius = egui::CornerRadius::same(6);
pub const FIELD_RADIUS: egui::CornerRadius = egui::CornerRadius::same(8);
pub const FLOATING_RADIUS: egui::CornerRadius = egui::CornerRadius::same(12);
pub const CONTROL: Color32 = Color32::from_rgba_premultiplied(7, 13, 24, 84);
pub const CONTROL_EDGE: Color32 = Color32::from_rgba_premultiplied(64, 78, 91, 144);
pub const OVERLAY_SHADOW: egui::epaint::Shadow = egui::epaint::Shadow {
    offset: [0, 4],
    blur: 12,
    spread: 0,
    color: Color32::from_black_alpha(72),
};

pub const MUTED: Color32 = Color32::from_rgb(184, 199, 214);
pub const LINE: Color32 = Color32::from_rgba_premultiplied(73, 92, 108, 136);
pub const CHROME: Color32 = Color32::TRANSPARENT;
pub const SIDEBAR: Color32 = Color32::TRANSPARENT;
pub const CANVAS: Color32 = Color32::from_rgba_premultiplied(4, 9, 17, 96);
pub const SELECTED: Color32 = Color32::from_rgba_premultiplied(36, 52, 69, 185);
pub const HOVER: Color32 = Color32::from_rgba_premultiplied(26, 41, 57, 162);
pub const FG: Color32 = Color32::from_rgb(240, 240, 240);

pub fn accent_ink(accent: Accent) -> Color32 {
    match accent {
        Accent::Cyan => Color32::from_rgb(138, 233, 250),
        Accent::Blue => Color32::from_rgb(137, 178, 255),
        Accent::White => Color32::from_gray(243),
        Accent::Black => Color32::from_gray(207),
    }
}

pub fn primary(label: &str) -> impl egui::Widget + '_ {
    primary_sized(label, egui::Vec2::ZERO)
}

pub fn primary_sized(label: &str, min_size: egui::Vec2) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| {
        let response = egui::Widget::ui(
            egui::Button::new(egui::RichText::new(label).strong()).min_size(min_size),
            ui,
        );
        let strength = if response.hovered() || response.has_focus() {
            0.85
        } else {
            0.65
        };
        button_glint(ui, response.rect, strength);
        response
    }
}

pub fn button_glint(ui: &egui::Ui, rect: egui::Rect, strength: f32) {
    if rect.width() > 8. && rect.height() > 6. {
        paint_sheen(
            ui.painter(),
            rect.shrink2(egui::vec2(4., 2.)),
            Color32::WHITE,
            strength,
        );
    }
}

pub fn sheen(painter: &egui::Painter, rect: egui::Rect, opacity: f32) {
    paint_sheen(painter, rect, Color32::from_rgb(214, 242, 255), opacity);
}

fn paint_sheen(painter: &egui::Painter, rect: egui::Rect, color: Color32, opacity: f32) {
    if !rect.is_positive() {
        return;
    }
    // Keep reflections close to the edge so adjacent surfaces share the backdrop.
    let center = rect.center().x;
    let half_width = (rect.width() * 0.35).min(260.);
    let top = rect.top();
    let bottom = top + rect.height().min(8.);
    let mut mesh = egui::Mesh::default();
    let highlight = color.linear_multiply(opacity);
    mesh.colored_vertex(egui::pos2(center - half_width, top), Color32::TRANSPARENT);
    mesh.colored_vertex(egui::pos2(center, top), highlight);
    mesh.colored_vertex(egui::pos2(center + half_width, top), Color32::TRANSPARENT);
    mesh.colored_vertex(egui::pos2(center, bottom), Color32::TRANSPARENT);
    mesh.add_triangle(0, 1, 3);
    mesh.add_triangle(1, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
}

pub fn window_glass(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0., CANVAS);
    glass_edge(painter, rect, 0.92);
    painter.rect_stroke(
        rect,
        0.,
        egui::Stroke::new(1., Color32::from_rgba_unmultiplied(165, 220, 245, 130)),
        egui::StrokeKind::Inside,
    );
}

pub fn glass_edge(painter: &egui::Painter, rect: egui::Rect, intensity: f32) {
    if rect.width() < 8. || rect.height() < 4. {
        return;
    }
    let rect = rect.shrink(1.);
    let light = Color32::from_rgba_unmultiplied(197, 232, 255, 142).linear_multiply(intensity);
    painter.hline(
        rect.x_range().shrink(2.),
        rect.top(),
        egui::Stroke::new(1., light),
    );
    painter.vline(
        rect.left(),
        rect.y_range().shrink(2.),
        egui::Stroke::new(1., light.linear_multiply(0.48)),
    );
    sheen(painter, rect.shrink2(egui::vec2(3., 0.)), intensity);
}

pub fn glass_inset() -> egui::Frame {
    egui::Frame::new()
        .fill(CONTROL)
        .stroke(egui::Stroke::new(1., CONTROL_EDGE))
        .corner_radius(FIELD_RADIUS)
}

pub fn apply(ctx: &egui::Context, accent: Accent) {
    let ink = accent_ink(accent);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = SIDEBAR;
    style.visuals.window_fill = CONTROL;
    style.visuals.extreme_bg_color = CANVAS;
    style.visuals.override_text_color = Some(FG);
    style.visuals.selection.bg_fill = SELECTED;
    style.visuals.selection.stroke = egui::Stroke::new(1., ink);
    style.visuals.hyperlink_color = ink;
    style.visuals.window_stroke =
        egui::Stroke::new(1., Color32::from_rgba_unmultiplied(177, 217, 241, 139));
    style.visuals.window_corner_radius = FLOATING_RADIUS;
    style.visuals.window_shadow = OVERLAY_SHADOW;
    style.visuals.menu_corner_radius = FLOATING_RADIUS;
    style.visuals.popup_shadow = OVERLAY_SHADOW;
    for (visual, fill, rim) in [
        (
            &mut style.visuals.widgets.inactive,
            Color32::TRANSPARENT,
            105,
        ),
        (
            &mut style.visuals.widgets.hovered,
            Color32::from_white_alpha(15),
            168,
        ),
        (
            &mut style.visuals.widgets.active,
            Color32::from_white_alpha(27),
            225,
        ),
        (
            &mut style.visuals.widgets.open,
            Color32::from_white_alpha(20),
            180,
        ),
    ] {
        visual.bg_fill = fill;
        visual.weak_bg_fill = fill;
        visual.bg_stroke = egui::Stroke::new(1., Color32::from_white_alpha(rim));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glass_tint_leaves_most_of_the_backdrop_visible() {
        assert!(
            CANVAS.a() <= 96,
            "glass tint obscures too much of the backdrop"
        );
    }
}
