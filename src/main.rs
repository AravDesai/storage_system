use eframe::egui::{
    self, Align2, Button, Color32, Context, FontFamily, FontId, Id, LayerId, Order, Pos2, Rect, Ui,
};
use egui_circle_trim::egui_circle_trim::CircleTrim;
use lb_rs::{File, Uuid};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use std::hash;
use std::hash::Hash;

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

        let data_clone = data.clone();
        let data_clone_two = data.clone();
        let data_clone_three = data.clone();
        

        let mut root = data[0].clone();

        for item in data {
            if item.file.id == item.file.parent {
                root = item;
            }
        }

        let root_hash = hash_data_converter(root);
        let root_hash_pass= root_hash.clone(); //needed to pass root value to self

        //let size_adjusted_data = vec![];


        
        let mut data_map = HashMap::new();

        
        let mut children = vec![];
        for item in data_clone {
            if item.file.parent == root_hash.id && item.file.id != root_hash.id {
                children.push(hash_data_converter(item));
            }
        }
        data_map.insert(root_hash, children.clone());

        let children_clone = children.clone();

        data_map = children_maker(data_map, children_clone.clone(), &data_clone_two);

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
            return HashData { id: data.file.id, parent: data.file.parent, name: data.file.name.clone(), size: data.size }
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

    pub fn get_color() -> Color32 {
        let colors = vec![
            Color32::BROWN,
            Color32::BLUE,
            Color32::RED,
            Color32::GREEN,
            Color32::YELLOW,
        ];
        return colors[rand::thread_rng().gen_range(0..colors.len())];
    }

    pub fn file_recurser(
        &self,
        ui: &mut Ui,
        mut layer_id: i32,
        parent: HashData,
        radius: f32,
        mut inner_bound: u64,
        outer_bound: u64,
        center: Pos2,
        view_type: ViewType,
    ) {
        let potential_children = self.data_map.get(&parent);
        match potential_children{
            Some(children) => for child in children{
                //println!("{:?}", child);
                let child_length = (child.size/parent.size) * outer_bound;
                let trim = CircleTrim{ color: Self::get_color(), inner_radius: radius, start_angle: inner_bound, end_angle: inner_bound + child_length, center, layer_id: LayerId { order: Order::PanelResizeLine, id: Id::new(layer_id) }, button_pressed: false, view_type };
                ui.add(trim);
                inner_bound+=child_length;
                //layer_id += 1;
            }
            None => return,
        }
        //println!("");
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
                            self.current_root.size.to_string() + " MB",
                            FontId {
                                size: 15.0,
                                family: FontFamily::Proportional,
                            },
                            Color32::BLACK,
                        );
                    ui.allocate_ui_at_rect(center_text, |ui| {
                        ui.label(self.current_root.size.to_string() + " MB")
                            .on_hover_text(self.current_root.name.to_string());
                    });
                });

                self.file_recurser(ui, 1, self.current_root.clone(), self.inner_radius, 0, 360, center.min, self.view_type);
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
                        self.current_root.size.to_string() + " MB",
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
                        ui.label(self.current_root.size.to_string() + " MB")
                            .on_hover_text(self.current_root.name.to_string());
                    },
                );

                self.file_recurser(ui, 1, self.current_root.clone(), self.inner_radius, 0, bottom.max.x as u64, bottom.min, self.view_type);

                // let slab = CircleTrim{ color: Color32::BLUE, inner_radius: 20.0, start_angle: 0, end_angle: bottom.max.x as u64, center: bottom.min, layer_id: LayerId{
                //     order: egui::Order::PanelResizeLine,
                //     id: Id::new(1),
                // }, button_pressed: false, view_type: self.view_type };

                // ui.add(slab);

            }



            //Recurse through files. For every file see how it fairs in terms of percentage on basis of size and start. make end new start for its fellow children

            //if this fails make thread make it run and maybe channel data like current root and repaint check back to it
        });
    }
}
