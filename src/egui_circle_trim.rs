pub mod egui_circle_trim {

    use std::f32::consts::PI;
    use std::vec;

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

    #[derive(Default)]
    pub struct CircleResponse{
        pub root_changed: bool,
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
<<<<<<< Updated upstream
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
=======
                    x: self.center.x + self.inner_radius
                    + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0).sin(),
                    y: self.center.y + self.inner_radius
                    + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0).cos(),
                },
                max: Pos2 {
                    x: self.center.x + self.inner_radius
                    + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0).sin(),
                    y: self.center.y + self.inner_radius
                    + ((((self.end_angle - self.start_angle) as f32) / 2.0) * PI / 180.0).cos(),
>>>>>>> Stashed changes
                },
            };
        }

<<<<<<< Updated upstream
        pub fn make_button(&self, ui: &mut Ui) -> Response {
            return ui
                .allocate_ui_at_rect(self.get_center_rect(), |ui| {
                    ui.add(Button::new("").fill(Color32::WHITE).rounding(100.0).small());
                })
                .response;
=======
        pub fn make_button(&self, ui: &mut Ui, out: &mut CircleResponse){
            // ui.ctx().input(|i|{
            //     for event in i.events{
            //         match event{
            //             eframe::egui::Event::Copy => todo!(),
            //             eframe::egui::Event::Cut => todo!(),
            //             eframe::egui::Event::Paste(_) => todo!(),
            //             eframe::egui::Event::Text(_) => todo!(),
            //             eframe::egui::Event::Key { key, physical_key, pressed, repeat, modifiers } => todo!(),
            //             eframe::egui::Event::PointerMoved(_) => todo!(),
            //             eframe::egui::Event::MouseMoved(_) => todo!(),
            //             eframe::egui::Event::PointerButton { pos, button, pressed, modifiers } => todo!(),
            //             eframe::egui::Event::PointerGone => todo!(),
            //             eframe::egui::Event::Zoom(_) => todo!(),
            //             eframe::egui::Event::Ime(_) => todo!(),
            //             eframe::egui::Event::Touch { device_id, id, phase, pos, force } => todo!(),
            //             eframe::egui::Event::MouseWheel { unit, delta, modifiers } => todo!(),
            //             eframe::egui::Event::WindowFocused(_) => todo!(),
            //             eframe::egui::Event::AccessKitActionRequest(_) => todo!(),
            //             eframe::egui::Event::Screenshot { viewport_id, image } => todo!(),
            //         }
            //     }})
            
            if ui
                .allocate_ui_at_rect(self.get_center_rect(), |ui| {
                    ui
                        .add(Button::new("").fill(Color32::WHITE).rounding(100.0).small())
                        .clicked()
                })
                .response.clicked(){
                    out.root_changed = true;
                }
>>>>>>> Stashed changes
        }

        //Sector of annulus
        pub fn paint_annulus_sector(&self, ui: &mut Ui) {
            let painter = ui.painter();
            let mut path_points = vec![];

            for i in (self.start_angle..self.end_angle).rev() {
                let angle = (i as f32 * PI) / 180.0;
                path_points.push(Pos2 {
                    x: self.center.x + ((self.inner_radius + 20.0) * angle.cos()),
                    y: self.center.y + ((self.inner_radius + 20.0) * angle.sin()),
                });
            }

<<<<<<< Updated upstream
            painter.add(PathShape {
=======
            for i in self.start_angle..self.end_angle {
                let angle = (i as f32 * PI) / 180.0;
                path_points.push(Pos2 {
                    x: self.center.x + (self.inner_radius * angle.cos()),
                    y: self.center.y + (self.inner_radius * angle.sin()),
                });
            }


            let print_points = path_points.iter().enumerate().map(|(i,p)|format!("({}, {}, {})", p.x, p.y, i)).collect::<Vec<String>>().join("\n");
            println!("{}", print_points);

            painter
            .clone()
            .with_layer_id(LayerId {
                order: eframe::egui::Order::PanelResizeLine,
                id: Id::new(self.layer),
            })
            .add(PathShape {
>>>>>>> Stashed changes
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

<<<<<<< Updated upstream
        fn add_contents(&mut self, ui: &mut Ui) -> eframe::egui::Response {
            let button_pos = self.get_center_rect();
=======
        pub fn draw(&mut self, ui: &mut Ui) -> CircleResponse {

            let mut output = CircleResponse::default();

>>>>>>> Stashed changes

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
<<<<<<< Updated upstream
                //self.make_button(ui);
=======
                self.make_button(ui, &mut output);
>>>>>>> Stashed changes
            }

            return output;
        }
    }
    // impl eframe::egui::Widget for CircleTrim {
    //     fn ui(mut self, ui: &mut Ui) -> eframe::egui::Response {
    //         self.add_contents(ui)
    //     }
    // }
}
