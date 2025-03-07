use color_art;
use colors_transform::{self, Color};
use data::NodeLayer;
use eframe::egui::{
    self, menu, Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Pos2, Rect, Rounding,
    Sense, Stroke, TextWrapMode, Ui,
};
use lb_rs::model::file_metadata::FileType;
use lb_rs::model::usage::bytes_to_human;
use lb_rs::Uuid;
use serde::Deserialize;
use std::hash::Hash;
use std::str::FromStr;
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

    pub fn get_color(&self, parent: Uuid, layer: u64, mut child_number: usize) -> Color32 {
        //change first number in gap depending on scheme
        let gap = 60.0 * 2.0;
        if layer == 1 {
            //60 gap
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

            //72 gap
            // let starting_colors = vec![
            //     Color32::from_hex(&(color_art::color!(HSL, 36.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            //     Color32::from_hex(&(color_art::color!(HSL, 108.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            //     Color32::from_hex(&(color_art::color!(HSL, 180.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            //     Color32::from_hex(&(color_art::color!(HSL, 252.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            //     Color32::from_hex(&(color_art::color!(HSL, 324.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            // ];

            //90 gap
            // let starting_colors = vec![
            //     Color32::from_hex(&(color_art::color!(HSL, 45.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            //     Color32::from_hex(&(color_art::color!(HSL, 135.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            //     Color32::from_hex(&(color_art::color!(HSL, 225.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            //     Color32::from_hex(&(color_art::color!(HSL, 315.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            // ];

            //120 gap
            // let starting_colors = vec![
            //     Color32::from_hex(&(color_art::color!(HSL, 60.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            //     Color32::from_hex(&(color_art::color!(HSL, 180.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            //     Color32::from_hex(&(color_art::color!(HSL, 300.0, 1.0, 0.5)).hex())
            //         .unwrap_or(Color32::DEBUG_COLOR),
            // ];

            if child_number >= starting_colors.len() {
                child_number = child_number % starting_colors.len();
            }
            return starting_colors[child_number];
        }

        let parent_color = self.colors.iter().find(|p| p.id == parent).unwrap();

        let child_fraction =
            (child_number as f32) / (self.data.get_children(&parent_color.id).len() as f32);

        let hue_difference = (gap / layer as f32) * child_fraction;

        let parent_hsl_color = colors_transform::Rgb::from(
            parent_color.color.r() as f32,
            parent_color.color.g() as f32,
            parent_color.color.b() as f32,
        )
        .to_hsl();

        let mut new_hue = parent_hsl_color.get_hue()
            + (hue_difference - ((1.0 / 2.0) * (gap / layer as f32))).round();

        if new_hue <= 0.0 {
            new_hue = 0.0;
        }
        if new_hue >= 360.0 {
            new_hue = 359.9;
        }

        let new_lum = (1.0 / layer as f32).max(0.5);

        return Color32::from_hex(
            &(color_art::color!(HSL, new_hue, 1.0 / layer as f32, new_lum)).hex(),
        )
        .unwrap_or(Color32::DEBUG_COLOR);
    }

    pub fn get_color_at_home(
        &self,
        curr_id: Uuid,
        mut layer: usize,
        mut child_number: usize,
    ) -> Color32 {
        let big_table = vec![
            //Red - orange - yellow
            [
                //l1
                Color32::from_hex("#991b1b").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#9a3412").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#92400e").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#854d0e").unwrap_or(Color32::DEBUG_COLOR),
                //l2
                Color32::from_hex("#b91c1c").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#c2410c").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#b45309").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#a16207").unwrap_or(Color32::DEBUG_COLOR),
                //l3
                Color32::from_hex("#dc2626").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#ea580c").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#d97706").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#ca8a04").unwrap_or(Color32::DEBUG_COLOR),
                //l4
                Color32::from_hex("#ef4444").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#f97316").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#f59e0b").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#eab308").unwrap_or(Color32::DEBUG_COLOR),
            ],
            //Green - blue
            [
                //l1
                Color32::from_hex("#065f46").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#115e59").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#155e75").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#075985").unwrap_or(Color32::DEBUG_COLOR),
                //l2
                Color32::from_hex("#047857").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#0f766e").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#0e7490").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#0369a1").unwrap_or(Color32::DEBUG_COLOR),
                //l3
                Color32::from_hex("#059669").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#0d9488").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#0891b2").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#0284c7").unwrap_or(Color32::DEBUG_COLOR),
                //l4
                Color32::from_hex("#10b981").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#14b8a6").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#06b6d4").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#0ea5e9").unwrap_or(Color32::DEBUG_COLOR),
            ],
            //blue - purple - pink
            [
                //l1
                Color32::from_hex("#1e40af").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#3730a3").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#5b21b6").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#6b21a8").unwrap_or(Color32::DEBUG_COLOR),
                //l2
                Color32::from_hex("#1d4ed8").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#4338ca").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#6d28d9").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#7e22ce").unwrap_or(Color32::DEBUG_COLOR),
                //l3
                Color32::from_hex("#2563eb").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#4f46e5").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#7c3aed").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#9333ea").unwrap_or(Color32::DEBUG_COLOR),
                //l4
                Color32::from_hex("#3b82f6").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#6366f1").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#8b5cf6").unwrap_or(Color32::DEBUG_COLOR),
                Color32::from_hex("#a855f7").unwrap_or(Color32::DEBUG_COLOR),
            ],
        ];
        if layer == 1 {
            if child_number > 2 {
                child_number = child_number % 3;
            }
            return big_table[child_number][0];
        }

        //could potentially get rid of these if statements
        if layer > 4 {
            layer = layer % 4;
        }

        if child_number > 3 {
            child_number = child_number % 4;
        }

        let parent_color = self
            .colors
            .iter()
            .find(|item| item.id == self.data.all_files.get(&curr_id).unwrap().file.parent)
            .unwrap()
            .color;

        let parent_type = big_table
            .iter()
            .enumerate()
            .find_map(|(row_index, row)| {
                row.iter()
                    .position(|&x| x == parent_color)
                    .map(|col_index| (row_index, col_index))
            })
            .unwrap()
            .0;

        let second_term = if child_number == 0 {
            (layer - 1) * 4
        } else {
            layer * child_number
        };
        return big_table[parent_type][second_term];
    }

    pub fn only_layers(&self, curr_id: Uuid, mut layer: usize, mut child_number: usize) -> Color32 {
        let big_table = vec![
            //red
            [
                Color32::from_rgb(128, 15, 47),
                Color32::from_rgb(164, 19, 60),
                Color32::from_rgb(201, 24, 74),
                Color32::from_rgb(255, 77, 109),
                Color32::from_rgb(255, 117, 143),
                Color32::from_rgb(255, 143, 163),
            ],
            //green
            [
                Color32::from_rgb(27, 67, 50),
                Color32::from_rgb(45, 106, 79),
                Color32::from_rgb(64, 145, 108),
                Color32::from_rgb(82, 183, 136),
                Color32::from_rgb(116, 198, 157),
                Color32::from_rgb(116, 198, 157),
            ],
            //blue
            [
                Color32::from_rgb(2, 62, 138),
                Color32::from_rgb(0, 119, 182),
                Color32::from_rgb(0, 150, 199),
                Color32::from_rgb(0, 180, 216),
                Color32::from_rgb(72, 202, 228),
                Color32::from_rgb(144, 224, 239),
            ],
        ];
        if layer == 1 {
            if child_number > 2 {
                child_number = child_number % 3;
            }
            return big_table[child_number][0];
        }

        let parent_color = self
            .colors
            .iter()
            .find(|item| item.id == self.data.all_files.get(&curr_id).unwrap().file.parent)
            .unwrap()
            .color;

        let parent_type = big_table
            .iter()
            .enumerate()
            .find_map(|(row_index, row)| {
                row.iter()
                    .position(|&x| x == parent_color)
                    .map(|col_index| (row_index, col_index))
            })
            .unwrap()
            .0;

        if layer > 4 {
            layer = layer % 4;
        }

        return big_table[parent_type][layer];
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
        let mut general_counter = 0;
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

            // let current_color = self
            //     .colors
            //     .iter()
            //     .find_map(|element| {
            //         if element.id == item.id {
            //             return Some(element.color);
            //         } else {
            //             return None;
            //         }
            //     })
            //     .unwrap_or(MyApp::get_color(
            //         &self,
            //         item_filerow.file.parent,
            //         current_layer,
            //         child_number,
            //     ));

            // let current_color = self
            //     .colors
            //     .iter()
            //     .find_map(|element| {
            //         if element.id == item.id {
            //             return Some(element.color);
            //         } else {
            //             return None;
            //         }
            //     })
            //     .unwrap_or(MyApp::get_color_at_home(
            //         &self,
            //         item.id,
            //         current_layer as usize,
            //         child_number - 1,
            //     ));

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
                .unwrap_or(MyApp::only_layers(
                    &self,
                    item.id,
                    current_layer as usize,
                    child_number - 1,
                ));

            let tab_intel: egui::WidgetText = egui::RichText::new(item.name.clone())
                .font(egui::FontId::monospace(12.0))
                .color(Color32::BLACK)
                .into();
            let tab_intel_galley = tab_intel.into_galley(
                ui,
                Some(TextWrapMode::Truncate),
                paint_rect.width(),
                egui::TextStyle::Body,
            );
            let tab_intel_rect = egui::Align2::LEFT_TOP
                .anchor_size(paint_rect.left_center(), tab_intel_galley.size());

            painter.clone().rect(
                paint_rect,
                Rounding::ZERO,
                current_color,
                Stroke {
                    width: 0.5,
                    color: Color32::BLACK,
                },
            );

            if paint_rect.width() >= 50.0 {
                ui.painter().galley(
                    tab_intel_rect.left_center() - egui::vec2(0.0, 5.5),
                    tab_intel_galley,
                    ui.visuals().text_color(),
                );
            }

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
