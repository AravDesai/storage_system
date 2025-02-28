use color_art;
use data::NodeLayer;
use eframe::egui::{
    self, menu, Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Pos2, Rect, Rounding,
    Sense, Stroke, Ui,
};
use lb_rs::model::file_metadata::FileType;
use lb_rs::model::usage::bytes_to_human;
use lb_rs::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self};
use std::hash::Hash;
use std::vec;
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

struct ColorHelper {
    id: Uuid,
    color: Color32,
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
    paint_order: Vec<NodeLayer>,
    colors: Vec<ColorHelper>,
}

impl MyApp {
    fn init(ctx: Context) -> Self {
        let json_info = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        let data: Vec<FileRow> = serde_json::from_str(&json_info).expect("Json not formatted well");

        //data::Data::get_children(&data::Data::init(), Uuid::parse_str("201a4ace-8c36-4eae-b982-181816a24b5d").as_ref().unwrap());
        println!("Made it through!");

        let mut root = data[0].clone(); //try cloning the id - error handling things

        for item in &data {
            if item.file.id == item.file.parent {
                root = item.clone();
            }
        }

        let root_cleaner = root.clone();

        let mut size_adjusted_data = vec![];

        for item in &data {
            if item.file.is_folder() {
                let new_size = size_finder(item.clone(), data.clone(), 0, root_cleaner.clone());
                if new_size == 0 {
                    continue;
                }
                size_adjusted_data.push(FileRow {
                    file: item.clone().file,
                    size: new_size,
                });
            }
            if item.file.is_document() {
                size_adjusted_data.push(item.clone());
            }
        }

        let size_to_hash = size_adjusted_data.clone();
        let size_to_hash_two = size_adjusted_data.clone();

        let mut cleaned_root = size_adjusted_data[0].clone();
        for item in size_adjusted_data {
            if item.file.id == item.file.parent {
                cleaned_root = item;
            }
        }

        let root_hash = hash_data_converter(cleaned_root);
        let root_hash_pass = root_hash.clone(); //needed to pass root value to self

        let mut folder_table = HashMap::new();

        let mut children = vec![];
        for item in size_to_hash {
            if item.file.parent == root_hash.id && item.file.id != root_hash.id {
                children.push(hash_data_converter(item));
            }
        }
        folder_table.insert(root_hash, children.clone());

        let children_clone = children.clone();

        folder_table = children_maker(folder_table, children_clone, &size_to_hash_two);

        pub fn children_maker(
            mut hashmap: HashMap<HashData, Vec<HashData>>,
            children: Vec<HashData>,
            data: &Vec<FileRow>,
        ) -> HashMap<HashData, Vec<HashData>> {
            for key in children {
                let mut spread = vec![];
                for item in data {
                    if item.file.parent == key.id {
                        spread.push(hash_data_converter(item.clone()));
                    }
                }
                if !spread.is_empty() {
                    let spread_clone = spread.clone();
                    hashmap.insert(key, spread);
                    hashmap = children_maker(hashmap, spread_clone, data)
                }
            }
            return hashmap;
        }

        pub fn hash_data_converter(data: FileRow) -> HashData {
            return HashData {
                id: data.file.id,
                parent: data.file.parent,
                name: data.file.name.clone(),
                size: data.size,
                file_type: data.file.file_type,
            };
        }

        //There's probably something more efficient than this - quickfix for now
        pub fn size_finder(
            subject: FileRow,
            dataset: Vec<FileRow>,
            mut size: u64,
            root: FileRow,
        ) -> u64 {
            let mut visited = vec![];
            for item in &dataset {
                if item.file.parent == subject.file.id
                    && item.file.id != root.file.id
                    && !visited.contains(item)
                {
                    visited.push(item.clone());
                    if item.file.is_folder() {
                        size += size_finder(item.clone(), dataset.clone(), 0, root.clone());
                    } else {
                        size += item.size;
                    }
                }
            }
            if visited.is_empty() && size == 0 {
                return 0;
            }
            return size;
        }

        Self {
            data: data,
            paint_order: vec![],
            layer_height: 20.0,
            colors: vec![],
        }
    }

    pub fn get_color(current_position: f32, layer: u64) -> Color32 {
        let mut filtered_position = current_position;
        if filtered_position > 360.0 {
            let factor = (filtered_position / 360.0) as u64;
            filtered_position -= (360 * factor) as f32;
        }
        let color = color_art::color!(HSL, filtered_position, 1.0 / (layer as f32), 0.5);
        let hex = color.hex();
        return egui::Color32::from_hex(&hex).unwrap_or(Color32::DEBUG_COLOR);
    }

    pub fn change_root(&mut self, new_root: Uuid) {
        self.data.current_root = new_root;
        self.paint_order = vec![];
    }

    pub fn reset_root(&mut self) {
        self.data.current_root = self.data.overall_root;
        self.paint_order = vec![];
    }

    pub fn follow_paint_order(&mut self, ui: &mut Ui, root_anchor: Rect) -> Option<Uuid> {
        let mut root_status: Option<Uuid> = None;
        let mut current_layer = 0;
        let mut current_position = 0.0;
        let mut general_counter = 1;
        let mut visited_folders: Vec<DrawHelper> = vec![];
        let mut current_parent = DrawHelper {
            id: self.data.current_root,
            starting_position: 0.0,
        };
        for item in &self.paint_order {
            let item_filerow = self.data.all_files.get(&item.id).unwrap();

            if current_layer != item.layer {
                current_position = 0.0;
                current_layer = item.layer;
            }

            if item_filerow.file.parent != current_parent.id {
                current_position = visited_folders
                    .iter()
                    .find(|parent| parent.id == item_filerow.file.parent)
                    .unwrap()
                    .starting_position;
                current_parent = DrawHelper {
                    id: self
                        .data
                        .all_files
                        .get(&item.id)
                        .unwrap()
                        .file
                        .parent
                        .clone(),
                    starting_position: current_position.clone(),
                };
            }
            let painter = ui.painter();
            let paint_rect = Rect {
                min: Pos2 {
                    x: current_position,
                    y: root_anchor.min.y - (current_layer as f32) * self.layer_height,
                },
                max: Pos2 {
                    x: current_position + (item.portion * root_anchor.max.x),
                    y: root_anchor.min.y - ((current_layer - 1) as f32) * self.layer_height,
                },
            };
            let current_color = self
                .colors
                .iter()
                .find_map(|element| {
                    if element.id == item.id {
                        return Some(element.color);
                    } else {
                        return None;
                    }
                })
                .unwrap_or(MyApp::get_color(current_position, current_layer));

            painter.clone().rect(
                paint_rect,
                Rounding::ZERO,
                current_color,
                Stroke {
                    width: 0.5,
                    color: Color32::BLACK,
                },
            );

            let display_size = if item_filerow.file.is_folder() {
                bytes_to_human(self.data.folder_sizes.get(&item.id).unwrap().clone())
            } else {
                bytes_to_human(item_filerow.size)
            };

            let response = ui.interact(paint_rect, Id::new(general_counter), Sense::click());

            if response.clicked() {
                if item_filerow.file.is_folder() {
                    root_status = Some(item.id.clone());
                }
            }

            response.on_hover_text(
                "Name:\n".to_owned()
                    + &self
                        .data
                        .all_files
                        .get(&item.id)
                        .unwrap()
                        .file
                        .name
                        .to_string()
                    + "\nSize:\n"
                    + &display_size,
            );

            if item_filerow.file.is_folder() {
                visited_folders.push(DrawHelper {
                    id: item.id.clone(),
                    starting_position: current_position.clone(),
                });
            }
            self.colors.push(ColorHelper {
                id: item.id,
                color: current_color,
            });
            current_position += item.portion * root_anchor.max.x;

            general_counter += 1;
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let window_size = ctx.input(|i: &egui::InputState| i.screen_rect());

            ui.menu_button("View Type", |ui| {
                if ui.button("Rectangular").clicked() {
                    self.view_type = ViewType::Rectangular;
                }
                if ui.button("Circular").clicked() {
                    self.view_type = ViewType::Circular;
                }
            });

            ui.heading("Storage Viewer");

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
                                self.current_root.size.to_string() + " B",
                                FontId {
                                    size: 15.0,
                                    family: FontFamily::Proportional,
                                },
                                Color32::BLACK,
                            );
                        ui.allocate_ui_at_rect(center_text, |ui| {
                            ui.label(self.current_root.size.to_string() + " B")
                                .on_hover_text(self.current_root.name.to_string());
                        });
                    });

            //Root drawing logic

            let root_draw_anchor = Rect {
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
                    bytes_to_human(*self.data.folder_sizes.get(&self.data.current_root).unwrap()),
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
                    ui.label(bytes_to_human(
                        *self.data.folder_sizes.get(&self.data.current_root).unwrap(),
                    ))
                    .on_hover_text(
                        self.data
                            .all_files
                            .get(&self.data.current_root)
                            .unwrap()
                            .file
                            .name
                            .to_string(),
                    );
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
                            self.current_root.size.to_string() + " B",
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
                            ui.label(self.current_root.size.to_string() + " B")
                                .on_hover_text(self.current_root.name.to_string());
                        },
                    );

                    // let mut trim = CircleTrim {
                    //     color: Self::get_color(0, 0),
                    //     inner_radius: 20.0,
                    //     start_angle: 0.0,
                    //     end_angle: 100.0,
                    //     layer_id: LayerId {
                    //         order: Order::PanelResizeLine,
                    //         id: Id::new(1),
                    //     },
                    //     button_pressed: false,
                    //     view_type: self.view_type,
                    //     center: bottom,
                    // };
                    // CircleTrim::paint_annulus_sector(&trim, ui);

                    // let mut brim = CircleTrim {
                    //     color: Self::get_color(0, 1),
                    //     inner_radius: 40.0,
                    //     start_angle: 0.0,
                    //     end_angle: 50.0,
                    //     layer_id: LayerId {
                    //         order: Order::PanelResizeLine,
                    //         id: Id::new(1),
                    //     },
                    //     button_pressed: false,
                    //     view_type: self.view_type,
                    //     center: bottom,
                    // };
                    // CircleTrim::paint_annulus_sector(&brim, ui);

                    // ui.allocate_ui_at_rect(trim.get_center_rect(), |ui| {
                    //     ui.with_layer_id(LayerId { order: Order::Debug, id: Id::new(1) }, |ui|{
                    //         ui.add(
                    //             Button::new("")
                    //                 .fill(Color32::DEBUG_COLOR)
                    //                 .rounding(100.0)
                    //                 .small(),
                    // )})});

                    match self.file_recurser(
                        ui,
                        1,
                        self.current_root.clone(),
                        self.inner_radius,
                        0.0,
                        bottom.max.x as u64,
                        bottom,
                        self.view_type,
                        0,
                        0,
                    ) {
                        Some(root) => {
                            println!("In the match!");
                            self.current_root = root;
                        }
                        None => {
                            return;
                        }
                    }
                }
            }
        });
    }
}
