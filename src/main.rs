use data::{FileRow, NodeLayer};
use eframe::egui::{
    self, Align2, Button, Color32, Context, FontFamily, FontId, Id, LayerId, Order, Pos2, Rect,
    Rounding, Stroke, TextBuffer, TextWrapMode, Ui, Vec2,
};
use egui_circle_trim::egui_circle_trim::{CircleResponse, CircleTrim};
//use lb_rs::model::file_metadata::FileType;
use lb_rs::shared::file_metadata::FileType;
use lb_rs::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self};
use std::hash::Hash;

mod data;
pub mod egui_circle_trim;

#[derive(Debug, Deserialize, Clone, Hash, PartialEq, Eq)]
struct HashData {
    id: Uuid,
    parent: Uuid,
    name: String,
    file_type: FileType,
    size: u64,
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum ViewType {
    Rectangular,
    Circular,
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 750.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Memory Viewer",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::init(cc.egui_ctx.clone())))),
    );
}

struct MyApp {
    data: data::Data,
    layer_height: u64,
    view_type: ViewType,
    paint_order: Vec<NodeLayer>,
}

impl MyApp {
    fn init(ctx: Context) -> Self {
        // let json_info = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        // let raw_data: Vec<FileRow> = serde_json::from_str(&json_info).expect("Json not formatted well");

        let data = data::Data::init(data::Data::from_file("parth-doc-data.json".to_owned()));

        Self {
            data: data,
            view_type: ViewType::Rectangular,
            paint_order: vec![],
            layer_height: 20,
        }
    }

    pub fn get_color(mut general: usize, mut specific: usize) -> Color32 {
        let colors = vec![
            [
                Color32::RED,
                Color32::DARK_RED,
                Color32::LIGHT_RED,
                Color32::from_rgb(255, 165, 0),
            ],
            [
                Color32::GREEN,
                Color32::DARK_GREEN,
                Color32::LIGHT_GREEN,
                Color32::from_rgb(0, 250, 154),
            ],
            [
                Color32::BLUE,
                Color32::DARK_BLUE,
                Color32::LIGHT_BLUE,
                Color32::from_rgb(123, 104, 238),
            ],
            [
                Color32::YELLOW,
                Color32::from_rgb(139, 128, 0),
                Color32::from_rgb(255, 245, 158),
                Color32::from_rgb(249, 166, 2),
            ],
        ];

        if general >= colors.len() {
            general = general % colors.len();
        }

        if specific >= colors[0].len() {
            specific = specific % colors[0].len();
        }
        return colors[general][specific];
    }

    pub fn follow_paint_order(&self, ui: &mut Ui, bottom: Rect ) {
        let mut current_layer = 0;
        let mut current_position = 0.0;
        let mut general_counter = 1;
        for item in &self.paint_order {
            if current_layer != item.layer {
                current_position = 0.0;
                current_layer = item.layer;
            }
            match self.view_type {
                ViewType::Rectangular => {
                    let painter = ui.painter();
                    let paint_rect = 
                        Rect {
                            min: Pos2 {
                                x: current_position,
                                y: bottom.min.y - ((current_layer + 1) as f32),
                            },
                            max: Pos2 {
                                x: current_position+item.portion,
                                y: bottom.min.y - (current_layer as f32),
                            },
                        };

                    painter.clone().rect(
                        paint_rect,
                        Rounding::ZERO,
                        MyApp::get_color(general_counter,current_layer as usize), //Ill make this look nicer soon,
                        Stroke {
                            width: 0.5,
                            color: Color32::BLACK,
                        },
                    );
                }
                ViewType::Circular => return, //will fill in logic here soon
            }
            current_position+=item.portion;
            general_counter+=1;
        }
    }

    // pub fn file_recurser(
    //     &self,
    //     ui: &mut Ui,
    //     layer_id: i32,
    //     parent: HashData,
    //     radius: f32,
    //     mut inner_bound: f32,
    //     outer_bound: u64,
    //     center: Rect,
    //     view_type: ViewType,
    //     mut general_color: usize,
    //     specific_color: usize,
    // ) -> Option<HashData> {
    //     let potential_children = self.folder_table.get(&parent);
    //     match potential_children {
    //         Some(children) => {
    //             for child in children {
    //                 let child_length =
    //                     (child.size as f32 / parent.size as f32) * outer_bound as f32;
    //                 let trim = CircleTrim {
    //                     color: Self::get_color(general_color, specific_color),
    //                     inner_radius: radius,
    //                     start_angle: inner_bound,
    //                     end_angle: inner_bound + child_length,
    //                     center,
    //                     layer_id: LayerId {
    //                         order: Order::PanelResizeLine,
    //                         id: Id::new(layer_id),
    //                     },
    //                     button_pressed: false,
    //                     view_type,
    //                 };
    //                 CircleTrim::paint_annulus_sector(&trim, ui);
    //                 if child.file_type == FileType::Folder {
    //                     ui.with_layer_id(
    //                         LayerId {
    //                             order: eframe::egui::Order::Foreground,
    //                             id: Id::new(1),
    //                         },
    //                         |ui| {
    //                             ui.allocate_ui_at_rect(trim.get_center_rect(), |ui| {
    //                                 let mut checker = false;
    //                                 if ui
    //                                     .add(
    //                                         Button::new("")
    //                                             .fill(Color32::WHITE)
    //                                             .rounding(100.0)
    //                                             .small(),
    //                                     )
    //                                     .on_hover_text(child.name.clone())
    //                                     .clicked()
    //                                 {
    //                                     println!("Name: {}", child.name);
    //                                     println!("Parent: {}", child.parent);
    //                                     println!("Size: {}", child.size);
    //                                     println!("Radius: {}", radius);
    //                                     println!("Layer: {}", layer_id);
    //                                     println!(
    //                                         "Color: {:?}",
    //                                         Self::get_color(general_color, specific_color)
    //                                     );
    //                                     println!("Rect: {}", trim.get_center_rect());
    //                                     println!("Child Length: {}", child_length);
    //                                     println!("Out: {}", outer_bound);
    //                                     println!("");
    //                                     checker = true;
    //                                     //return Some(child);
    //                                 }
    //                                 if checker {
    //                                     println!("Inside if");
    //                                     println!("{:?}", Some(child));
    //                                     return Some(child);
    //                                 } else {
    //                                     return None;
    //                                 }
    //                             })
    //                         },
    //                     );
    //                     self.file_recurser(
    //                         ui,
    //                         layer_id + 9,
    //                         child.clone(),
    //                         radius + 20.0,
    //                         inner_bound,
    //                         (child_length) as u64,
    //                         center,
    //                         view_type,
    //                         general_color,
    //                         specific_color + 1,
    //                     );
    //                 }
    //                 inner_bound += child_length;
    //                 general_color += 1;
    //             }
    //             return None;
    //         }
    //         None => return None,
    //     }
    // }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //Start of pre ui checks
            if self.paint_order.is_empty() {
                self.paint_order = data::Data::get_paint_order(&self.data);
            }

            //Allows for dynamic window
            let window_size = ctx.input(|i: &egui::InputState| i.screen_rect());

            //Top buttons
            ui.menu_button("View Type", |ui| {
                if ui.button("Rectangular").clicked() {
                    self.view_type = ViewType::Rectangular;
                }
                if ui.button("Circular").clicked() {
                    self.view_type = ViewType::Circular;
                }
            });

            ui.menu_button("Reset Root", |ui| {
                if ui.button("Confirm Reset").clicked() {
                    self.data.reset_root();
                    self.paint_order = vec![];
                }
            });

            //Start of UI
            ui.heading("Storage Viewer");

            //Insert logic for painting everything but root here or in each match before roots are painted

            match self.view_type {
                ViewType::Circular => {
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
                        painter
                            .clone()
                            .with_layer_id(LayerId {
                                order: egui::Order::Foreground,
                                id: Id::new(1),
                            })
                            .circle(
                                window_size.max / 2.0,
                                50.0,
                                Color32::WHITE,
                                egui::Stroke {
                                    width: 0.0,
                                    color: Color32::from_rgb(255, 255, 255),
                                },
                            );

                        painter
                            .clone()
                            .with_layer_id(LayerId {
                                order: egui::Order::Debug,
                                id: Id::new(1),
                            })
                            .text(
                                center.min,
                                Align2::CENTER_CENTER,
                                self.data
                                    .folder_sizes
                                    .get(&self.data.current_root)
                                    .unwrap()
                                    .to_string()
                                    + " B",
                                FontId {
                                    size: 15.0,
                                    family: FontFamily::Proportional,
                                },
                                Color32::BLACK,
                            );
                        ui.allocate_ui_at_rect(center_text, |ui| {
                            ui.label(
                                self.data
                                    .folder_sizes
                                    .get(&self.data.current_root)
                                    .unwrap()
                                    .to_string()
                                    + " B",
                            )
                            .on_hover_text(
                                self.data
                                    .all_files
                                    .get(&self.data.current_root)
                                    .unwrap()
                                    .file
                                    .name
                                    .to_string()
                                    .to_string(),
                            );
                        });
                    });

                    // self.file_recurser(
                    //     ui,
                    //     1,
                    //     self.current_root.clone(),
                    //     self.inner_radius,
                    //     0.0,
                    //     360,
                    //     center,
                    //     self.view_type,
                    //     0,
                    //     0,
                    // );
                }
                ViewType::Rectangular => {
                    let bottom = Rect {
                        min: Pos2 {
                            x: 0.0,
                            y: window_size.max.y - 40.0,
                        },
                        max: window_size.max,
                    };

                    let bottom_text = Rect {
                        min: Pos2 {
                            x: bottom.max.x / 2.0,
                            y: bottom.max.y - 15.0,
                        },
                        max: window_size.max,
                    };

                    self.follow_paint_order(ui, bottom);
                    let painter = ui.painter();
                    painter
                        .clone()
                        .with_layer_id(LayerId {
                            order: egui::Order::Foreground,
                            id: Id::new(1),
                        })
                        .rect_filled(bottom, 0.0, Color32::WHITE);

                    painter
                        .clone()
                        .with_layer_id(LayerId {
                            order: egui::Order::Debug,
                            id: Id::new(2),
                        })
                        .text(
                            bottom_text.min,
                            Align2::CENTER_BOTTOM,
                            self.data
                                .folder_sizes
                                .get(&self.data.current_root)
                                .unwrap()
                                .to_string()
                                + " B",
                            FontId {
                                size: 15.0,
                                family: FontFamily::Proportional,
                            },
                            Color32::BLACK,
                        );
                    ui.allocate_ui_at_rect(
                        Rect {
                            min: Pos2 {
                                x: bottom_text.min.x - 30.0,
                                y: bottom_text.min.y - 15.0,
                            },
                            max: bottom_text.max,
                        },
                        |ui| {
                            ui.label(
                                self.data
                                    .folder_sizes
                                    .get(&self.data.current_root)
                                    .unwrap()
                                    .to_string()
                                    + " B",
                            )
                            .on_hover_text(
                                self.data
                                    .all_files
                                    .get(&self.data.current_root)
                                    .unwrap()
                                    .file
                                    .name
                                    .to_string(),
                            );
                        },
                    );

                    // match self.file_recurser(
                    //     ui,
                    //     1,
                    //     self.current_root.clone(),
                    //     self.inner_radius,
                    //     0.0,
                    //     bottom.max.x as u64,
                    //     bottom,
                    //     self.view_type,
                    //     0,
                    //     0,
                    // ) {
                    //     Some(root) => {
                    //         println!("In the match!");
                    //         self.current_root = root;
                    //     }
                    //     None => {
                    //         return;
                    //     }
                    // }
                }
            }
        });
    }
}
