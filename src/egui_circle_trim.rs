pub mod egui_circle_trim {

    use std::f32::consts::PI;

    use eframe::egui::layers::ShapeIdx;
    use eframe::egui::{
        Button, Color32, Context, Id, Label, LayerId, Painter, Pos2, Rect, Response, Rounding,
        Stroke, Ui, Vec2, WidgetText,
    };

    use eframe::epaint::tessellator::{path, Path};
    use eframe::epaint::{PathShape, PathStroke};

    pub struct CircleTrim {
        color: Color32,
        inner_radius: f32,
        start_angle: i32,
        end_angle: i32,
        center: Pos2,
    }

    impl CircleTrim {
        pub fn new(
            color: Color32,
            inner_radius: f32,
            start_angle: i32,
            end_angle: i32,
            center: Pos2,
        ) -> Self {
            Self {
                color,
                inner_radius,
                start_angle,
                end_angle,
                center,
            }
        }

        pub fn get_center_rect(&self) -> Rect {
            return Rect {
                min: Pos2 {
                    x: self.center.x + self.inner_radius - 12.0
                        + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0).sin(),
                    y: self.center.y + self.inner_radius - 12.0
                        + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0).cos(),
                },
                max: Pos2 {
                    x: self.center.x + self.inner_radius - 12.0
                        + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0).sin(),
                    y: self.center.y + self.inner_radius - 12.0
                        + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0).cos(),
                },
            };
        }

        pub fn make_button(&self, ui: &mut Ui) -> Response {
            return ui
                .allocate_ui_at_rect(self.get_center_rect(), |ui| {
                    ui.add(Button::new("").fill(Color32::WHITE).rounding(100.0).small());
                })
                .response;
        }

        //Sector of annulus
        pub fn paint_annulus_sector(&self, ui: &mut Ui) {
            let painter = ui.painter();
            let mut path_points = vec![];

            for i in self.start_angle..self.end_angle {
                let angle = (i as f32 * PI) / 180.0;
                path_points.push(Pos2 {
                    x: self.center.x + (self.inner_radius * angle.sin()),
                    y: self.center.y + (self.inner_radius * angle.cos()),
                });
            }

            for i in (self.start_angle..self.end_angle).rev() {
                let angle = (i as f32 * PI) / 180.0;
                path_points.push(Pos2 {
                    x: self.center.x + ((self.inner_radius + 20.0) * angle.sin()),
                    y: self.center.y + ((self.inner_radius + 20.0) * angle.cos()),
                });
            }

            painter.add(PathShape {
                points: path_points,
                closed: true,
                fill: Color32::BLUE,
                stroke: PathStroke {
                    width: 1.0,
                    color: eframe::epaint::ColorMode::Solid(self.color),
                },
            });

            //painter.rect_filled(self.get_center_rect(), Rounding::ZERO, Color32::WHITE);
        }

        fn add_contents(&mut self, ui: &mut Ui) -> eframe::egui::Response {
            let button_pos = self.get_center_rect();

            let desired_size = Vec2 {
                x: button_pos.width(),
                y: button_pos.height(),
            };

            // let desired_size = Vec2{
            //     x: self.center.x,
            //     y: self.center.y,
            // };

            let (rect, response) =
                ui.allocate_exact_size(desired_size, eframe::egui::Sense::click());

            if ui.is_rect_visible(rect) {
                self.paint_annulus_sector(ui);
                //self.make_button(ui);
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
