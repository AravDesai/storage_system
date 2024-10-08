use eframe::egui::{
    self, Align2, Button, Color32, Context, FontFamily, FontId, Id, LayerId, Order, Pos2, Rect, TextWrapMode, Ui, Vec2
};
use egui_circle_trim::egui_circle_trim::{CircleResponse, CircleTrim};
use lb_rs::{File, Uuid};
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::collections::HashMap;
use std::hash::Hash;
use lb_rs::FileType;

pub mod egui_circle_trim;

#[derive(Debug, Deserialize, Clone, Hash, PartialEq, Eq)]
struct Data {
    file: File,
    size: u64,
}

#[derive(Debug, Deserialize, Clone, Hash, PartialEq, Eq)]
struct HashData{
    id: Uuid,
    parent: Uuid,
    name: String,
    file_type: FileType,
    size: u64
}

#[derive(PartialEq, Clone, Copy)]
enum ViewType {
    Rectangular,
    Circular,
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
    data_map: HashMap<HashData, Vec<HashData>>,
    current_root: HashData,
    inner_radius: f32,
    start_angle: i32,
    repaint_check: bool,
    view_type: ViewType,
}

impl MyApp {
    fn init(ctx: Context) -> Self {
        let json_info = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        let data: Vec<Data> = serde_json::from_str(&json_info).expect("Json not formatted well");

        let data_clone_two = data.clone();
        let data_clone_three = data.clone();

        let mut root = data[0].clone();

        for item in data {
            if item.file.id == item.file.parent {
                root = item;
            }
        }

        let root_cleaner = root.clone();

        let mut size_adjusted_data = vec![];

        for item in data_clone_two{
            if item.file.is_folder(){
                let new_size = size_finder(item.clone(), data_clone_three.clone(), 0, root_cleaner.clone());
                if new_size == 0 {
                    continue;
                }
                size_adjusted_data.push(Data { file: item.clone().file, size: new_size });
            }
            if item.file.is_document(){
                size_adjusted_data.push(item);
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
        let root_hash_pass= root_hash.clone(); //needed to pass root value to self

        let mut data_map = HashMap::new();

        
        let mut children = vec![];
        for item in size_to_hash {
            if item.file.parent == root_hash.id && item.file.id != root_hash.id {
                children.push(hash_data_converter(item));
            }
        }
        data_map.insert(root_hash, children.clone());

        let children_clone = children.clone();

        data_map = children_maker(data_map, children_clone, &size_to_hash_two);

        pub fn children_maker(mut hashmap: HashMap<HashData, Vec<HashData>>, children: Vec<HashData>, data: &Vec<Data>) -> HashMap<HashData, Vec<HashData>>{
            for key in children{
                let mut spread = vec![];
                for item in data{
                    if item.file.parent == key.id {
                        spread.push(hash_data_converter(item.clone()));
                    }
                }
                if !spread.is_empty(){
                    let spread_clone = spread.clone();
                    hashmap.insert(key, spread);
                    hashmap = children_maker(hashmap, spread_clone, data)
                }
            }
            return hashmap;
        }

        pub fn hash_data_converter(data: Data)->HashData{
            return HashData { id: data.file.id, parent: data.file.parent, name: data.file.name.clone(), size: data.size, file_type: data.file.file_type }
        }

        //There's probably something more efficient than this - quickfix for now
        pub fn size_finder(subject: Data, dataset: Vec<Data>, mut size: u64, root: Data) -> u64 {
            let mut visited = vec![];
            for item in &dataset {
                if item.file.parent == subject.file.id && item.file.id != root.file.id && !visited.contains(item) {
                    visited.push(item.clone());
                    if item.file.is_folder() {
                        size += size_finder(item.clone(), dataset.clone(), 0, root.clone());
                    } else {
                        size += item.size;
                    }
                }
            }
            if visited.is_empty() && size == 0{
                return 0;
            }
            return size;
        }

        Self {
            data_map: data_map,
            current_root: root_hash_pass,
            inner_radius: 50.0,
            start_angle: 0,
            repaint_check: true,
            view_type: ViewType::Rectangular,
        }

        
    }

    pub fn get_color(mut general: usize, mut specific: usize) -> Color32 {
        if general > 3 {
            general = general%4; 
        }
        if specific > 2 {
            specific = specific % 3; 
        }
        let colors = vec![[Color32::RED, Color32::DARK_RED, Color32::LIGHT_RED],[Color32::GREEN, Color32::DARK_GREEN, Color32::LIGHT_GREEN],[Color32::BLUE, Color32::DARK_BLUE, Color32::LIGHT_BLUE], [Color32::YELLOW, Color32::from_rgb(139, 128, 0), Color32::from_rgb(255, 245, 158)]];
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
        center: Pos2,
        view_type: ViewType,
        mut general_color: usize,
        specific_color: usize,
    ) {
        let potential_children = self.data_map.get(&parent);
        match potential_children{
            Some(children) => for child in children{
                let child_length = (child.size as f32/parent.size as f32) * outer_bound as f32;
                let mut trim = CircleTrim{ color: Self::get_color(general_color,specific_color), inner_radius: radius, start_angle: inner_bound, end_angle: inner_bound + child_length, center, layer_id: LayerId { order: Order::PanelResizeLine, id: Id::new(layer_id) }, button_pressed: false, view_type };
                CircleTrim::paint_annulus_sector(&trim, ui);
                if child.file_type == FileType::Folder {
                    ui.with_layer_id(
                        LayerId {
                            order: eframe::egui::Order::Debug,
                            id: Id::new(1),
                        },
                        |ui| {
                            ui.allocate_ui_at_rect(trim.get_center_rect(), |ui| {
                                if ui
                                    .add(Button::new("").fill(Color32::WHITE).rounding(100.0).small())
                                    .clicked()
                                {
                                    println!("{}", child.name);
                                }
                            })
                        },
                    );
                    //CircleTrim::make_button(&mut trim, ui, &mut CircleResponse{ root_changed: false});
                    self.file_recurser(ui, layer_id + 1, child.clone(), radius+20.0, inner_bound, (inner_bound + child_length) as u64, center, view_type, general_color, specific_color + 1);
                }
                inner_bound+=child_length;
                general_color+=1;
            }
            None => return,
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

            if self.view_type == ViewType::Circular {
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

                self.file_recurser(ui, 1, self.current_root.clone(), self.inner_radius, 0.0, 360, center.min, self.view_type, 0, 0);
            }

            if self.view_type == ViewType::Rectangular {
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

                // let slab = CircleTrim{ color: Color32::BLUE, inner_radius: 20.0, start_angle: 30.0, end_angle: 60.0 , center: bottom.min, layer_id: LayerId{
                //         order: egui::Order::PanelResizeLine,
                //         id: Id::new(1),
                //     }, button_pressed: false, view_type: self.view_type };
                //     CircleTrim::paint_annulus_sector(&slab, ui);
                //     ui.with_layer_id(
                //         LayerId {
                //             order: eframe::egui::Order::Foreground,
                //             id: Id::new(1),
                //         },
                //         |ui| {
                //             ui.allocate_ui_at_rect(slab.get_center_rect(), |ui| {
                //                 if ui
                //                     .add(Button::new("").fill(Color32::WHITE).rounding(100.0).small())
                //                     .clicked()
                //                 {
                //                     println!("Clicked!");
                //                 }
                //             })
                //         },
                //     );

                self.file_recurser(ui, 1, self.current_root.clone(), self.inner_radius, 0.0, bottom.max.x as u64, bottom.min, self.view_type,0,0);
            }
        });
    }
}
