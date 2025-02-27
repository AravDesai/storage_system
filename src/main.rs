use data::NodeLayer;
use eframe::egui::{
    self, menu, Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Pos2,
    Rect, Rounding, Stroke, Ui,
};
use lb_rs::model::usage::bytes_to_human;
use lb_rs::model::file_metadata::FileType;
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
}

impl MyApp {
    fn init(_ctx: Context) -> Self {
        let data = data::Data::init(data::Data::from_file("parth-doc-data.json".to_owned()));

        Self {
            data: data,
            paint_order: vec![],
            layer_height: 20.0,
        }
    }

    pub fn get_color(mut general: usize, mut specific: usize) -> Color32 {
        //Need to change this so it becomes more consistent when clicking through different roots
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
            painter.clone().rect(
                paint_rect,
                Rounding::ZERO,
                MyApp::get_color(general_counter, current_layer as usize), //I'll make this look nicer soon,
                Stroke {
                    width: 0.5,
                    color: Color32::BLACK,
                },
            );

            let display_size = if item_filerow.file.is_folder() {
                bytes_to_human(self.data
                    .folder_sizes
                    .get(&item.id)
                    .unwrap()
                    .clone()) 
            } else {
                bytes_to_human(item_filerow.size)
            };

            ui.allocate_ui_at_rect(paint_rect, |ui| {
                ui.with_layer_id(
                    LayerId {
                        order: eframe::egui::Order::Foreground,
                        id: Id::new(3),
                    },
                    |ui| {
                        if ui
                            .colored_label(Color32::WHITE, ".")
                            .on_hover_text(
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
                                    + &display_size
                                    + "\nParent:\n"
                                    + &self
                                        .data
                                        .all_files
                                        .get(
                                            &self.data.all_files.get(&item.id).unwrap().file.parent,
                                        )
                                        .unwrap()
                                        .file
                                        .name
                                        .to_string(),
                            )
                            .clicked()
                        {
                            if item_filerow.file.is_folder() {
                                root_status = Some(item.id.clone());
                            }
                        }
                    },
                )
            });
            if item_filerow.file.is_folder() {
                visited_folders.push(DrawHelper {
                    id: item.id.clone(),
                    starting_position: current_position.clone(),
                });
            }
            current_position += item.portion * root_anchor.max.x;

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
                    bytes_to_human(*self.data
                        .folder_sizes
                        .get(&self.data.current_root)
                        .unwrap()),
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
                        bytes_to_human(*self.data
                            .folder_sizes
                            .get(&self.data.current_root)
                            .unwrap()),
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
            let potential_new_root = self.follow_paint_order(ui, root_draw_anchor);
            match potential_new_root {
                Some(_) => self.change_root(potential_new_root.unwrap()),
                None => (),
            }
        });
    }
}
