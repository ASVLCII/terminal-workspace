use crate::surfaces::{CONTROL_RADIUS, button_glint};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
#[derive(Clone, Copy)]
pub enum Icon {
    Terminal,
    Folder,
    Search,
    Chat,
    Settings,
    Columns,
    Image,
    Plus,
    Close,
    Maximize,
    #[cfg(windows)]
    Minimize,
    #[cfg(windows)]
    Restore,
    Paperclip,
    Chevron(bool),
}
pub fn paint(p: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let r = Rect::from_center_size(rect.center(), egui::vec2(16., 16.));
    let at = |x: f32, y: f32| Pos2::new(r.left() + x, r.top() + y);
    let s = Stroke::new(1.3, color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([at(a.0, a.1), at(b.0, b.1)], s);
    };
    match icon {
        Icon::Terminal => {
            p.rect_stroke(r.shrink(1.), 2., s, egui::StrokeKind::Inside);
            line((4., 5.), (7., 8.));
            line((7., 8.), (4., 11.));
            line((9., 11.), (12., 11.));
        }
        Icon::Folder => {
            p.add(egui::Shape::closed_line(
                vec![
                    at(1., 4.),
                    at(6., 4.),
                    at(8., 6.),
                    at(15., 6.),
                    at(15., 14.),
                    at(1., 14.),
                ],
                s,
            ));
        }
        Icon::Search => {
            p.circle_stroke(at(6.5, 6.5), 4.5, s);
            line((10., 10.), (14.5, 14.5));
        }
        Icon::Chat => {
            p.add(egui::Shape::closed_line(
                vec![
                    at(2., 2.),
                    at(14., 2.),
                    at(14., 11.),
                    at(7., 11.),
                    at(3., 15.),
                    at(3., 11.),
                    at(2., 11.),
                ],
                s,
            ));
        }
        Icon::Settings => {
            p.circle_stroke(at(8., 8.), 5., s);
            p.circle_stroke(at(8., 8.), 2., s);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.;
                line(
                    (8. + 5. * a.cos(), 8. + 5. * a.sin()),
                    (8. + 7. * a.cos(), 8. + 7. * a.sin()),
                );
            }
        }
        Icon::Columns => {
            p.rect_stroke(r.shrink(1.), 1., s, egui::StrokeKind::Inside);
            line((8., 2.), (8., 14.));
        }
        Icon::Image => {
            p.rect_stroke(r.shrink(1.), 2., s, egui::StrokeKind::Inside);
            p.circle_stroke(at(5., 5.), 1.2, s);
            line((2., 13.), (7., 8.));
            line((7., 8.), (10., 11.));
            line((10., 11.), (12., 9.));
            line((12., 9.), (15., 13.));
        }
        Icon::Plus => {
            line((8., 2.), (8., 14.));
            line((2., 8.), (14., 8.));
        }
        Icon::Close => {
            line((3., 3.), (13., 13.));
            line((13., 3.), (3., 13.));
        }
        Icon::Maximize => {
            p.rect_stroke(r.shrink(2.), 0., s, egui::StrokeKind::Inside);
        }
        #[cfg(windows)]
        Icon::Minimize => line((3., 9.), (13., 9.)),
        #[cfg(windows)]
        Icon::Restore => {
            line((6., 2.), (14., 2.));
            line((14., 2.), (14., 10.));
            p.rect_stroke(
                Rect::from_min_max(at(2., 6.), at(11., 14.)),
                0.,
                s,
                egui::StrokeKind::Inside,
            );
        }
        Icon::Chevron(open) => {
            if open {
                line((3., 6.), (8., 11.));
                line((8., 11.), (13., 6.));
            } else {
                line((6., 3.), (11., 8.));
                line((11., 8.), (6., 13.));
            }
        }
        Icon::Paperclip => {
            p.add(egui::Shape::line(
                vec![
                    at(5., 12.),
                    at(11., 5.),
                    at(9., 3.),
                    at(2., 11.),
                    at(3., 14.),
                    at(6., 15.),
                    at(14., 6.),
                    at(14., 3.),
                    at(11., 1.),
                    at(8., 2.),
                ],
                s,
            ));
        }
    }
}
pub fn button(ui: &mut egui::Ui, icon: Icon, label: &str) -> egui::Response {
    let response = ui
        .add_sized([30., 30.], egui::Button::new("").frame(false))
        .on_hover_text(label);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
    });
    if response.hovered() {
        ui.painter()
            .rect_filled(response.rect, CONTROL_RADIUS, Color32::from_white_alpha(15));
        button_glint(ui, response.rect, 0.55);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect,
            CONTROL_RADIUS,
            ui.visuals().selection.stroke,
            egui::StrokeKind::Inside,
        );
    }
    paint(
        ui.painter(),
        response.rect,
        icon,
        if response.hovered() {
            Color32::WHITE
        } else {
            crate::surfaces::MUTED
        },
    );
    response
}
pub fn inline(ui: &mut egui::Ui, icon: Icon, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(18., 20.), egui::Sense::hover());
    paint(ui.painter(), rect, icon, color);
}
