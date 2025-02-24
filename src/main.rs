use data::NodeLayer;
use eframe::egui::{
    self, Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Pos2, Rect,
    Rounding, Stroke, Ui,
};
//use lb_rs::model::file_metadata::FileType;
use lb_rs::shared::file_metadata::FileType;
use lb_rs::Uuid;
use serde::Deserialize;
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
    layer_height: f32,
    view_type: ViewType,
    paint_order: Vec<NodeLayer>,
}

impl MyApp {
    fn init(_ctx: Context) -> Self {
        // let json_info = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        // let raw_data: Vec<FileRow> = serde_json::from_str(&json_info).expect("Json not formatted well");

        let data = data::Data::init(data::Data::from_file("parth-doc-data.json".to_owned()));

        Self {
            data: data,
            view_type: ViewType::Rectangular,
            paint_order: vec![],
            layer_height: 20.0,
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

    pub fn change_root(&mut self, new_root: Uuid){
        self.data.current_root = new_root;
        self.paint_order = vec![];
    }

    pub fn reset_root(&mut self){
        self.data.current_root = self.data.overall_root;
        self.paint_order = vec![];
    }

    pub fn follow_paint_order(&mut self, ui: &mut Ui, bottom: Rect) -> Option<Uuid>{
        let mut root_status: Option<Uuid> = None;
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
                    let paint_rect = Rect {
                        min: Pos2 {
                            x: current_position,
                            y: bottom.min.y - (current_layer as f32) * self.layer_height,
                        },
                        max: Pos2 {
                            x: current_position + (item.portion * bottom.max.x),
                            y: bottom.min.y - ((current_layer - 1) as f32) * self.layer_height,
                        },
                    };
                    painter.clone().rect(
                        paint_rect,
                        Rounding::ZERO,
                        MyApp::get_color(general_counter, current_layer as usize), //I'll make this look nicer soon,
                        Stroke {
                            width: 0.5,
                            color: Color32::BLACK,
                        },
                    );
                    ui.allocate_ui_at_rect(paint_rect, |ui| {
                        ui.with_layer_id(
                            LayerId {
                                order: eframe::egui::Order::Foreground,
                                id: Id::new(1),
                            },
                            |ui| {
                                if ui.colored_label(Color32::WHITE, ".").on_hover_text(
                                    self.data
                                        .all_files
                                        .get(&item.id)
                                        .unwrap()
                                        .file
                                        .name
                                        .to_string()
                                        + "\nParent:\n"
                                        + &self
                                            .data
                                            .all_files
                                            .get(
                                                &self
                                                    .data
                                                    .all_files
                                                    .get(&item.id)
                                                    .unwrap()
                                                    .file
                                                    .parent,
                                            )
                                            .unwrap()
                                            .file
                                            .name
                                            .to_string(),
                                ).clicked()  {
                                    if self.data.all_files.get(&item.id).unwrap().file.is_folder(){
                                        root_status = Some(item.id.clone());
                                    }
                                }
                            },
                        )
                    });
                }
                ViewType::Circular => return root_status, //will fill in logic here soon
            }
            current_position += item.portion * bottom.max.x;
            general_counter += 1;
        }
        return root_status;
    }

    //Comments will be deleted when circle logic is put in

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
                    self.reset_root();
                    self.paint_order = vec![];
                }
            });

            ui.heading("Storage Viewer");

            //Root drawing logic

            let mut root_draw_anchor = Rect{
                min: Pos2 { x: 0.0, y: 0.0 },
                max: Pos2 { x: 0.0, y: 0.0 },
            };
            
            match self.view_type {
                ViewType::Circular => {
                    root_draw_anchor = Rect {
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

                    ui.allocate_ui_at_rect(root_draw_anchor, |ui| {
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
                                root_draw_anchor.min,
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
                }
                ViewType::Rectangular => {
                    root_draw_anchor = Rect {
                        min: Pos2 {
                            x: 0.0,
                            y: window_size.max.y - 40.0,
                        },
                        max: window_size.max,
                    };

                    let bottom_text = Rect {
                        min: Pos2 {
                            x: root_draw_anchor.max.x / 2.0,
                            y: root_draw_anchor.max.y - 15.0,
                        },
                        max: window_size.max,
                    };

                    let painter = ui.painter();
                    painter
                        .clone()
                        .with_layer_id(LayerId {
                            order: egui::Order::Foreground,
                            id: Id::new(1),
                        })
                        .rect_filled(root_draw_anchor, 0.0, Color32::WHITE);

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
                }
            }
            let potential_new_root = self.follow_paint_order(ui, root_draw_anchor);
            match potential_new_root {
                Some(_) => self.change_root(potential_new_root.unwrap()),
                None => (),
            }
        });
    }
}
