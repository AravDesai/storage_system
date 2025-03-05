use color_art;
use colors_transform::{self, Color};
use data::NodeLayer;
use eframe::egui::{
    self, menu, Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Pos2, Rect, Rounding,
    Sense, Stroke, Ui,
};
use eframe::epaint::color;
use lb_rs::model::file_metadata::FileType;
use lb_rs::model::usage::bytes_to_human;
use lb_rs::Uuid;
use serde::Deserialize;
use std::hash::Hash;
mod data;

#[derive(Debug, Deserialize, Clone, Hash, PartialEq, Eq)]
struct HashData {
    id: Uuid,
    parent: Uuid,
    name: String,
    file_type: FileType,
    size: u64,
}

#[derive(Debug)]
struct DrawHelper {
    id: Uuid,
    starting_position: f32,
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
    fn init(_ctx: Context) -> Self {
        let data = data::Data::init(data::Data::from_file("parth-doc-data.json".to_owned()));

        Self {
            data: data,
            paint_order: vec![],
            layer_height: 50.0,
            colors: vec![],
        }
    }

    // pub fn get_color(current_position: f32, layer: u64) -> Color32 {
    //     let mut filtered_position = current_position;
    //     if filtered_position > 360.0 {
    //         let factor = (filtered_position / 360.0) as u64;
    //         filtered_position -= (360 * factor) as f32;
    //     }
    //     let color = color_art::color!(HSL, filtered_position, 1.0 / (layer as f32), 0.5);
    //     let hex = color.hex();
    //     return egui::Color32::from_hex(&hex).unwrap_or(Color32::DEBUG_COLOR);
    // }

    pub fn get_color(&self, parent: Uuid, layer: u64, mut child_number: usize) -> Color32 {
        if layer == 1 {
            let starting_colors = vec![
                Color32::from_hex(&(color_art::color!(HSL, 30.0, 1.0, 0.5)).hex())
                    .unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex(&(color_art::color!(HSL, 90.0, 1.0, 0.5)).hex())
                    .unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex(&(color_art::color!(HSL, 150.0, 1.0, 0.5)).hex())
                    .unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex(&(color_art::color!(HSL, 210.0, 1.0, 0.5)).hex())
                    .unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex(&(color_art::color!(HSL, 270.0, 1.0, 0.5)).hex())
                    .unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex(&(color_art::color!(HSL, 330.0, 1.0, 0.5)).hex())
                    .unwrap_or(Color32::DEBUG_COLOR),
            ];
            if child_number >= starting_colors.len() {
                child_number = child_number % starting_colors.len();
            }
            return starting_colors[child_number];
        }

        let parent_color = self.colors.iter().find(|p| p.id == parent).unwrap();

        let child_fraction =
            (child_number as f32) / (self.data.get_children(&parent_color.id).len() as f32);

        let hue_difference = (120.0 / layer as f32) * child_fraction;

        let parent_hsl_color = colors_transform::Rgb::from(
            parent_color.color.r() as f32,
            parent_color.color.g() as f32,
            parent_color.color.b() as f32,
        )
        .to_hsl();

        let parent_hue = parent_hsl_color.get_hue();
        println!("Parent hue: {:?}", parent_hue);
        println!("Hue diff: {:?}", hue_difference);
        println!("Layer: {:?}", layer);

        let mut new_hue = parent_hsl_color.get_hue()
            + (hue_difference - ((1.0 / 2.0) * (120.0 / layer as f32))).round();

        if new_hue < 0.0 {
            new_hue = 0.0;
        }
        if new_hue > 360.0 {
            new_hue = 360.0;
        }

        println!("New hue: {:?}\n", new_hue);

        return Color32::from_hex(
            &(color_art::color!(HSL, new_hue, 1.0 / layer as f32, 0.5)).hex(),
        )
        .unwrap_or(Color32::DEBUG_COLOR);
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
        let mut child_number = 1;
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
                child_number = 1;
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
                .unwrap_or(MyApp::get_color(
                    &self,
                    item_filerow.file.parent,
                    current_layer,
                    child_number,
                ));
            //.unwrap_or(MyApp::get_color(current_position, current_layer));

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
            child_number += 1;
            general_counter += 1;
        }
        return root_status;
    }
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

            ui.with_layer_id(
                LayerId {
                    order: egui::Order::Debug,
                    id: Id::new(1),
                },
                |ui| {
                    menu::bar(ui, |ui| {
                        if ui.button("Reset Root").clicked() {
                            self.reset_root();
                            self.paint_order = vec![];
                        }

                        ui.menu_button("Layer Size", |ui| {
                            ui.add(egui::Slider::new(&mut self.layer_height, 1.0..=100.0));
                        });
                    });
                },
            );

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
                },
            );
            let potential_new_root = self.follow_paint_order(ui, root_draw_anchor);
            match potential_new_root {
                Some(_) => self.change_root(potential_new_root.unwrap()),
                None => (),
            }
        });
    }
}
