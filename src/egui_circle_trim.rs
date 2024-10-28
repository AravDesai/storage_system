pub mod egui_circle_trim {

    use std::f32::consts::PI;
    use std::vec;

    use eframe::egui::layers::ShapeIdx;
    use eframe::egui::{
        Button, Color32, Context, Id, Label, LayerId, Painter, Pos2, Rect, Response, Rounding,
        Stroke, Ui, Vec2, WidgetText,
    };

    use eframe::egui_glow::painter;
    use eframe::epaint::tessellator::{path, Path};
    use eframe::epaint::{PathShape, PathStroke};

    use crate::ViewType;

    #[derive(Clone, Copy)]
    pub struct CircleTrim {
        pub color: Color32,
        pub inner_radius: f32,
        pub start_angle: f32,
        pub end_angle: f32,
        pub center: Rect,
        pub layer_id: LayerId,
        pub button_pressed: bool,
        pub(crate) view_type: ViewType,
    }

    #[derive(Default)]
    pub struct CircleResponse {
        pub root_changed: bool,
    }

    impl CircleTrim {
        pub fn new(
            color: Color32,
            inner_radius: f32,
            start_angle: f32,
            end_angle: f32,
            center: Rect,
            layer_id: LayerId,
            button_pressed: bool,
            view_type: ViewType,
        ) -> Self {
            Self {
                color,
                inner_radius,
                start_angle,
                end_angle,
                center,
                layer_id,
                button_pressed,
                view_type,
            }
        }

        pub fn get_center_rect(&self) -> Rect {
            if self.view_type == ViewType::Circular {
                return Rect {
                    min: Pos2 {
                        x: self.center.min.x + self.inner_radius - 12.0
                            + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0)
                                .sin(),
                        y: self.center.min.y + self.inner_radius - 12.0
                            + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0)
                                .cos(),
                    },
                    max: Pos2 {
                        x: self.center.min.x + self.inner_radius - 12.0
                            + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0)
                                .sin(),
                        y: self.center.min.y + self.inner_radius - 12.0
                            + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0)
                                .cos(),
                    },
                };
            }
            if self.view_type == ViewType::Rectangular {

                return Rect {
                    min: Pos2 {
                        x: self.center.min.x + ((self.start_angle + self.end_angle) as f32)/2.0 - 5.0,
                        y: self.center.max.y - self.inner_radius - 35.0,
                    },
                    max: Pos2 {
                        x: self.center.min.x + ((self.start_angle + self.end_angle) as f32)/2.0 - 5.0,
                        y: self.center.max.y - self.inner_radius - 30.0,
                    },
                };
                // return Rect {
                //     min: Pos2 {
                //         x: self.center.min.x + ((self.start_angle + self.end_angle) / 2.0),
                //         y: self.center.max.y - self.inner_radius,
                //     },
                //     max: Pos2 {
                //         x: self.center.min.x + ((self.start_angle + self.end_angle) / 2.0),
                //         y: self.center.max.y - self.inner_radius,
                //     },
                // };
            }
            panic!("Invalid ViewType");
        }

        pub fn status(&self) -> bool {
            //println!("{}", self.button_pressed);
            self.button_pressed
        }

        pub fn make_button(&mut self, ui: &mut Ui, out: &mut CircleResponse) {
            ui.with_layer_id(
                LayerId {
                    order: eframe::egui::Order::Debug,
                    id: Id::new(1),
                },
                |ui| {
                    ui.allocate_ui_at_rect(self.get_center_rect(), |ui| {
                        if ui
                            .add(Button::new("").fill(Color32::WHITE).rounding(100.0).small())
                            .clicked()
                        {
                            println!("Clicked!");
                            self.button_pressed = true;
                            out.root_changed = true;
                        }
                    })
                },
            );
        }

        //Sector of annulus
        pub fn paint_annulus_sector(&self, ui: &mut Ui) {
            if self.view_type == ViewType::Circular {
                let painter = ui.painter();
                let mut path_points = vec![];

                for i in self.start_angle as u32..self.end_angle as u32 {
                    let angle = (i as f32 * PI) / 180.0;
                    path_points.push(Pos2 {
                        x: self.center.min.x + (self.inner_radius * angle.sin()),
                        y: self.center.min.y + (self.inner_radius * angle.cos()),
                    });
                }

                for i in (self.start_angle as u32..self.end_angle as u32).rev() {
                    let angle = (i as f32 * PI) / 180.0;
                    path_points.push(Pos2 {
                        x: self.center.min.x + ((self.inner_radius + 20.0) * angle.sin()),
                        y: self.center.min.y + ((self.inner_radius + 20.0) * angle.cos()),
                    });
                }

                painter.clone().with_layer_id(self.layer_id).add(PathShape {
                    points: path_points,
                    closed: true,
                    fill: self.color,
                    stroke: PathStroke {
                        width: 1.0,
                        color: eframe::epaint::ColorMode::Solid(self.color),
                    },
                });

                //painter.rect_filled(self.get_center.min_rect(), Rounding::ZERO, Color32::WHITE);
            }

            if self.view_type == ViewType::Rectangular {
                let painter = ui.painter();
                let paint_rect = Rect {
                    min: Pos2 {
                        x: self.center.min.x + self.start_angle as f32,
                        y: self.center.max.y - self.inner_radius - 40.0,
                    },
                    max: Pos2 {
                        x: self.center.min.x + self.end_angle as f32,
                        y: self.center.max.y - self.inner_radius - 20.0,
                    },
                };
                painter.clone().with_layer_id(self.layer_id).rect(
                    paint_rect,
                    Rounding::ZERO,
                    self.color,
                    Stroke {
                        width: 0.5,
                        color: Color32::BLACK,
                    },
                );
            }
        }

        fn add_contents(&mut self, ui: &mut Ui) -> eframe::egui::Response {
            let button_pos = self.get_center_rect();

            let desired_size = Vec2 {
                x: button_pos.width(),
                y: button_pos.height(),
            };

            // let desired_size = Vec2{
            //     x: self.center.min.x,
            //     y: self.center.min.y,
            // };

            let (rect, response) =
                ui.allocate_exact_size(desired_size, eframe::egui::Sense::click());

            if ui.is_rect_visible(rect) {
                self.paint_annulus_sector(ui);
                self.make_button(
                    ui,
                    &mut CircleResponse {
                        root_changed: false,
                    },
                );
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
