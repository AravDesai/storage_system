use data::FileRow;
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
    folder_table: HashMap<HashData, Vec<HashData>>,
    current_root: HashData,
    inner_radius: f32,
    view_type: ViewType,
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
            folder_table: folder_table,
            current_root: root_hash_pass,
            inner_radius: 20.0,
            view_type: ViewType::Rectangular,
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

    pub fn file_recurser(
        &self,
        ui: &mut Ui,
        layer_id: i32,
        parent: HashData,
        radius: f32,
        mut inner_bound: f32,
        outer_bound: u64,
        center: Rect,
        view_type: ViewType,
        mut general_color: usize,
        specific_color: usize,
    ) -> Option<HashData> {
        let potential_children = self.folder_table.get(&parent);
        match potential_children {
            Some(children) => {
                for child in children {
                    let child_length =
                        (child.size as f32 / parent.size as f32) * outer_bound as f32;
                    let trim = CircleTrim {
                        color: Self::get_color(general_color, specific_color),
                        inner_radius: radius,
                        start_angle: inner_bound,
                        end_angle: inner_bound + child_length,
                        center,
                        layer_id: LayerId {
                            order: Order::PanelResizeLine,
                            id: Id::new(layer_id),
                        },
                        button_pressed: false,
                        view_type,
                    };
                    CircleTrim::paint_annulus_sector(&trim, ui);
                    if child.file_type == FileType::Folder {
                        ui.with_layer_id(
                            LayerId {
                                order: eframe::egui::Order::Foreground,
                                id: Id::new(1),
                            },
                            |ui| {
                                ui.allocate_ui_at_rect(trim.get_center_rect(), |ui| {
                                    let mut checker = false;
                                    if ui
                                        .add(
                                            Button::new("")
                                                .fill(Color32::WHITE)
                                                .rounding(100.0)
                                                .small(),
                                        )
                                        .on_hover_text(child.name.clone())
                                        .clicked()
                                    {
                                        println!("Name: {}", child.name);
                                        println!("Parent: {}", child.parent);
                                        println!("Size: {}", child.size);
                                        println!("Radius: {}", radius);
                                        println!("Layer: {}", layer_id);
                                        println!(
                                            "Color: {:?}",
                                            Self::get_color(general_color, specific_color)
                                        );
                                        println!("Rect: {}", trim.get_center_rect());
                                        println!("Child Length: {}", child_length);
                                        println!("Out: {}", outer_bound);
                                        println!("");
                                        checker = true;
                                        //return Some(child);
                                    }
                                    if checker {
                                        println!("Inside if");
                                        println!("{:?}", Some(child));
                                        return Some(child);
                                    } else {
                                        return None;
                                    }
                                })
                            },
                        );
                        self.file_recurser(
                            ui,
                            layer_id + 9,
                            child.clone(),
                            radius + 20.0,
                            inner_bound,
                            (child_length) as u64,
                            center,
                            view_type,
                            general_color,
                            specific_color + 1,
                        );
                    }
                    inner_bound += child_length;
                    general_color += 1;
                }
                return None;
            }
            None => return None,
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

                    self.file_recurser(
                        ui,
                        1,
                        self.current_root.clone(),
                        self.inner_radius,
                        0.0,
                        360,
                        center,
                        self.view_type,
                        0,
                        0,
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
