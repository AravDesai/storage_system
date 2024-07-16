pub mod egui_circle_trim{

use std::f32::consts::PI;

use eframe::egui::layers::ShapeIdx;
use eframe::egui::{Button, Color32, Context, Id, Label, LayerId, Painter, Pos2, Rect, Stroke, Ui, Vec2, WidgetText};

use eframe::epaint::tessellator::{path, Path};
use eframe::epaint::{PathShape, PathStroke};

pub struct CircleTrim{
    pub(crate) color: Color32,
    pub(crate) inner_radius: f32,
    //outer_radius: f32,
    pub(crate) ctx: Context,
    pub(crate) layer_id: LayerId,
    pub(crate) id: Id,
    pub(crate) rect: Rect,
    painter: Painter,
    start_angle: i32,
    end_angle: i32,
    center: Pos2,
}

impl CircleTrim{
    pub fn new(
        color: Color32,
        inner_radius: f32,
        ctx: Context,
        layer_id: LayerId,
        id: Id,
        rect: Rect,
        painter: Painter,
        start_angle: i32,
        end_angle: i32,
        center: Pos2,
    )-> Self{
        Self { color: color, inner_radius: inner_radius, ctx: ctx, layer_id: layer_id, id: id, rect: rect, painter: painter, start_angle: start_angle, end_angle: end_angle, center:center }
    }

    //Sector of annulus
    pub fn paint_annulus_sector(&self){
            let mut path_points = vec![];

            for i in self.start_angle..self.end_angle{
                let angle = (i as f32*PI)/180.0;
                path_points.push(Pos2{
                    x: self.center.x + (self.inner_radius * angle.sin()),
                    y: self.center.y + (self.inner_radius * angle.cos()),
                });
            }

            self.painter.add(PathShape{
                points: path_points,
                closed: false,
                fill: Color32::TRANSPARENT,
                stroke: PathStroke{
                    width: 20.0,
                    color: eframe::epaint::ColorMode::Solid(self.color),
                },
            });
    }

    //Hoverable Text functions:

    pub fn show_tooltip_ui(&self, add_contents: impl FnOnce(&mut Ui)) {
        crate::egui::containers::show_tooltip_for(
            &self.ctx,
            self.layer_id,
            self.id,
            &self.rect,
            add_contents,
        );
    }

    pub fn on_hover_ui(self, add_contents: impl FnOnce(&mut Ui)) -> Self {
        self.show_tooltip_ui(add_contents);
        self
    }

    #[doc(alias = "tooltip")]
    pub fn on_hover_text(self, text: impl Into<WidgetText>) -> Self {
        self.on_hover_ui(|ui| {
            ui.add(crate::egui::widgets::Label::new(text));
        })
    }

    fn add_contents(&mut self, ui: &mut Ui) -> eframe::egui::Response {
        let desired_size = Vec2{
            x: self.center.x,
            y: self.center.y,
        };
        let (rect, response) = ui.allocate_exact_size( desired_size, eframe::egui::Sense::click());

        //response.widget_info(|| egui::WidgetInfo::slider(self.value, &self.text));

        if ui.is_rect_visible(rect) {
            self.paint_annulus_sector();
        }

        response
    }

}
impl eframe::egui::Widget for CircleTrim {
    fn ui(mut self, ui: &mut Ui) -> eframe::egui::Response {
        self.add_contents(ui)
    }
}
}
