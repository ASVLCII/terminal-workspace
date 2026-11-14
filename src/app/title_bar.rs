use crate::{
    icons::{self, Icon},
    surfaces,
};
use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Rect, ResizeDirection, Sense, ViewportCommand,
    WidgetInfo, WidgetType, pos2, vec2,
};

pub(super) fn show(ui: &mut egui::Ui) {
    let maximized = ui.input(|input| input.viewport().maximized == Some(true));
    if !maximized && ui.is_enabled() {
        resize_edges(ui);
    }
    egui::Panel::top("window-title")
        .exact_size(40.)
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            let bar = ui.max_rect();
            surfaces::glass_edge(ui.painter(), bar, 0.92);
            let row = bar.shrink2(vec2(8., 5.));
            let control_width = 38.;
            let drag = Rect::from_min_max(
                row.min,
                pos2(row.right() - 3. * control_width, row.bottom()),
            );
            let response = ui.interact(drag, ui.id().with("window-drag"), Sense::click_and_drag());
            if response.double_clicked() {
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::Maximized(!maximized));
            } else if response.drag_started_by(egui::PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            }

            let painter = ui.painter().with_clip_rect(drag);
            let center = drag.center().y;
            icons::paint(
                &painter,
                Rect::from_center_size(pos2(drag.left() + 13., center), vec2(16., 16.)),
                Icon::Terminal,
                surfaces::FG,
            );
            painter.text(
                pos2(drag.left() + 28., center),
                Align2::LEFT_CENTER,
                "Terminal Workspace",
                FontId::proportional(15.),
                surfaces::FG,
            );

            for (index, (icon, label, command)) in [
                (Icon::Minimize, "Minimize", ViewportCommand::Minimized(true)),
                (
                    if maximized {
                        Icon::Restore
                    } else {
                        Icon::Maximize
                    },
                    if maximized {
                        "Restore window"
                    } else {
                        "Maximize"
                    },
                    ViewportCommand::Maximized(!maximized),
                ),
                (Icon::Close, "Close window", ViewportCommand::Close),
            ]
            .into_iter()
            .enumerate()
            {
                let left = drag.right() + index as f32 * control_width;
                let rect =
                    Rect::from_min_size(pos2(left, row.top()), vec2(control_width, row.height()));
                let response = ui.interact(rect, ui.id().with(label), Sense::click());
                response.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), label)
                });
                if response.hovered() || response.has_focus() {
                    let fill = Color32::from_white_alpha(15);
                    ui.painter()
                        .rect_filled(rect, surfaces::CONTROL_RADIUS, fill);
                    surfaces::button_glint(ui, rect, 0.55);
                }
                if response.has_focus() {
                    ui.painter().rect_stroke(
                        rect,
                        surfaces::CONTROL_RADIUS,
                        ui.visuals().selection.stroke,
                        egui::StrokeKind::Inside,
                    );
                }
                icons::paint(ui.painter(), rect, icon, surfaces::FG);
                if response.on_hover_text(label).clicked() {
                    ui.ctx().send_viewport_cmd(command);
                }
            }
        });
}

fn resize_edges(ui: &mut egui::Ui) {
    use ResizeDirection::*;
    let rect = ui.max_rect();
    let edge = 5.;
    let inner = rect.shrink(edge);
    for (min, max, direction, cursor) in [
        (rect.min, inner.min, NorthWest, CursorIcon::ResizeNwSe),
        (
            pos2(inner.right(), rect.top()),
            pos2(rect.right(), inner.top()),
            NorthEast,
            CursorIcon::ResizeNeSw,
        ),
        (
            pos2(rect.left(), inner.bottom()),
            pos2(inner.left(), rect.bottom()),
            SouthWest,
            CursorIcon::ResizeNeSw,
        ),
        (inner.max, rect.max, SouthEast, CursorIcon::ResizeNwSe),
        (
            pos2(inner.left(), rect.top()),
            pos2(inner.right(), inner.top()),
            North,
            CursorIcon::ResizeVertical,
        ),
        (
            pos2(inner.left(), inner.bottom()),
            pos2(inner.right(), rect.bottom()),
            South,
            CursorIcon::ResizeVertical,
        ),
        (
            pos2(rect.left(), inner.top()),
            pos2(inner.left(), inner.bottom()),
            West,
            CursorIcon::ResizeHorizontal,
        ),
        (
            pos2(inner.right(), inner.top()),
            pos2(rect.right(), inner.bottom()),
            East,
            CursorIcon::ResizeHorizontal,
        ),
    ] {
        let response = ui
            .interact(
                Rect::from_min_max(min, max),
                ui.id().with(direction as u8),
                Sense::drag(),
            )
            .on_hover_cursor(cursor);
        if response.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::BeginResize(direction));
        }
    }
}
