use eframe::egui::{
    self, Align2, Button, Color32, Context, FontFamily, FontId, Id, LayerId, Pos2, Rect,
};
use egui_circle_trim::egui_circle_trim::CircleTrim;
use lb_rs::{Config, Core, File, Uuid};
use serde::{Deserialize, Serialize};
use std::fs;

pub mod egui_circle_trim;

#[derive(Debug, Deserialize, Clone)]
struct Data {
    file: File,
    size: u64,
}

struct Tree {
    data: Data,
    children: Vec<Data>,
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 500.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Memory Viewer",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::init(cc.egui_ctx.clone())))),
    );
}

struct MyApp {
    current_root: Data,
    inner_radius: f32,
    start_angle: i32,
    data: Vec<Data>,
}

impl MyApp {
    fn init(ctx: Context) -> Self {
        let json_info = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        let data: Vec<Data> = serde_json::from_str(&json_info).expect("Json not formatted well");
        let data_clone = data.clone();
        let mut root = data[0].clone();
        for item in data {
            if item.file.id == item.file.parent {
                root = item;
            }
        }

        let mut sorted = vec![];
        sorted.push(root);

        Self {
            data: data_clone,
            current_root: sorted[0].clone(),
            inner_radius: 50.0,
            start_angle: 0,
        }
    }
<<<<<<< Updated upstream
}

fn file_recurser(parent: Data, data: Vec<Data>, root: Data) {
    let mut files = vec![];
    for file in data {
        if (file.file.parent == parent.file.id && file.file.id != root.file.id) {
            files.push(file);
=======

    fn file_recurser(&self, ui: &mut Ui,layer_id: i32, parent: Data, radius: f32, mut inner_bound: i32, outer_bound: i32, center: Pos2) {
        for file in &self.data {
            if file.file.parent == parent.file.id && file.file.id != self.current_root.file.id {
                let chunk = ((file.size/parent.size) * 360) as i32;
                let trim = CircleTrim::new(Color32::BLUE, radius, inner_bound, inner_bound + chunk, center, layer_id);
                
                ui.add(trim);

                if file.file.is_folder(){
                    self.file_recurser(ui, layer_id + 1, file.clone(), radius + 20.0, inner_bound, inner_bound + chunk, center);
                }   

                inner_bound+= chunk;
            }
>>>>>>> Stashed changes
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let window_size = ctx.input(|i: &egui::InputState| i.screen_rect());

            ui.heading("Memory Viewer");

            let center = Rect {
                min: window_size.max / 2.0,
                max: window_size.max,
            };

            let center_text = Rect {
                min: Pos2 {
                    x: (window_size.max.x / 2.0) - 25.0,
                    y: (window_size.max.y / 2.0) - 5.0,
                },
                max: window_size.max,
            };
            ui.allocate_ui_at_rect(center, |ui| {
                let painter = ui.painter();
                // painter
                //     .clone()
                //     .with_layer_id(LayerId {
                //         order: egui::Order::PanelResizeLine,
                //         id: Id::new(2),
                //     })
                //     .circle(
                //         window_size.max / 2.0,
                //         50.0,
                //         Color32::WHITE,
                //         egui::Stroke {
                //             width: 0.0,
                //             color: Color32::from_rgb(255, 255, 255),
                //         },
                //     );

                painter
                    .clone()
                    .with_layer_id(LayerId {
                        order: egui::Order::PanelResizeLine,
                        id: Id::new(4),
                    })
                    .circle(
                        window_size.max / 2.0,
                        70.0,
                        Color32::TRANSPARENT,
                        egui::Stroke {
                            width: 2.0,
                            color: Color32::from_rgb(255, 0, 0),
                        },
                    );

                painter
                    .clone()
                    .with_layer_id(LayerId {
                        order: egui::Order::Foreground,
                        id: Id::new(1),
                    })
                    .text(
                        center.min,
                        Align2::CENTER_CENTER,
                        self.current_root.size.to_string() + " MB",
                        FontId {
                            size: 15.0,
                            family: FontFamily::Proportional,
                        },
                        Color32::BLACK,
                    );
                ui.allocate_ui_at_rect(center_text, |ui| {
                    ui.label(self.current_root.file.name.to_string())
                        .on_hover_text(self.current_root.file.name.to_string());
                });
                let trim = CircleTrim::new(Color32::BLUE, self.inner_radius, 0, 90, center.min);

<<<<<<< Updated upstream
                ui.allocate_ui_at_rect(CircleTrim::get_center_rect(&trim), |ui| {
                    ui.with_layer_id(
                        LayerId {
                            order: egui::Order::Foreground,
                            id: Id::new(1),
                        },
                        |ui| {
                            ui.add(Button::new("").fill(Color32::WHITE).rounding(100.0).small())
                                .on_hover_text("Test")
                        },
                    )
                });
                ui.add(trim);
                file_recurser(
                    self.current_root.clone(),
                    self.data.clone(),
                    self.current_root.clone(),
                );
=======
                //ui.add(CircleTrim::new(Color32::RED, 50.0, 0, 90, center.min, 4));

                let mut blue = CircleTrim::new(Color32::BLUE, 50.0, 0, 90, center.min, 20);

                if blue.draw(ui).root_changed{

                }


                // ui.allocate_ui_at_rect(CircleTrim::get_center_rect(&trim), |ui| {
                //     ui.with_layer_id(
                //         LayerId {
                //             order: egui::Order::Foreground,
                //             id: Id::new(1),
                //         },
                //         |ui| {
                //             ui.add(Button::new("").fill(Color32::WHITE).rounding(100.0).small())
                //                 .on_hover_text("Test")
                //         },
                //     )
                // });

                // if (self.repaint_check) {
                //     self.file_recurser(ui, 2, self.current_root.clone(), 50.0, 0, 360, center.min);
                //     self.repaint_check = false;
                // }
>>>>>>> Stashed changes
            });
        });
    }
}
