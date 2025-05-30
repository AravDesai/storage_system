use color_art;
use colors_transform::{self, Color};
use data::NodeLayer;
use eframe::egui::{
    self, menu, Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Pos2, Rect, Rounding,
    Sense, Stroke, TextWrapMode, Ui,
};
use lb_rs::model::usage::bytes_to_human;
use lb_rs::Uuid;
mod data;

//Responsible for tracking on screen locations for folders
#[derive(Debug)]
struct DrawHelper {
    id: Uuid,
    starting_position: f32,
}

//Responsible for keeping colors consistent
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
        //Will be accepting real data here soon
        let data = data::Data::init(data::Data::from_file("parth-doc-data.json".to_owned()));

        Self {
            data: data,
            paint_order: vec![],
            layer_height: 50.0,
            colors: vec![],
        }
    }

    pub fn change_root(&mut self, new_root: Uuid) {
        self.data.current_root = new_root;
        self.paint_order = vec![];
    }

    pub fn reset_root(&mut self) {
        self.data.current_root = self.data.overall_root;
        self.paint_order = vec![];
    }

    pub fn get_color(&self, curr_id: Uuid, mut layer: usize, mut child_number: usize) -> Color32 {
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

        layer -= 1;

        if layer > 5 {
            layer = layer % 6;
        }

        return big_table[parent_type][layer];
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
                    item.id,
                    current_layer as usize,
                    child_number - 1,
                ));

            //Folder text logic
            let tab_intel: egui::WidgetText = egui::RichText::new(item.name.clone())
                .font(egui::FontId::monospace(12.0))
                .color({
                    let hsl_color = colors_transform::Rgb::from(
                        current_color.r().into(),
                        current_color.g().into(),
                        current_color.b().into(),
                    )
                    .to_hsl();
                    let mut luminance = 0.5;
                    if hsl_color.get_lightness() > 50.0 {
                        luminance = (hsl_color.get_lightness() - 50.0) / 100.0;
                    } else {
                        luminance = (hsl_color.get_lightness() + 50.0) / 100.0;
                    }
                    Color32::from_hex(
                        &(color_art::color!(
                            HSL,
                            hsl_color.get_hue(),
                            hsl_color.get_saturation() / 100.0,
                            luminance
                        ))
                        .hex(),
                    )
                    .unwrap_or(Color32::DEBUG_COLOR)
                })
                .into();
            let tab_intel_galley = tab_intel.into_galley(
                ui,
                Some(TextWrapMode::Truncate),
                paint_rect.width() - 5.0,
                egui::TextStyle::Body,
            );
            let tab_intel_rect = egui::Align2::LEFT_TOP.anchor_size(
                Pos2 {
                    x: paint_rect.left_center().x + 5.0,
                    y: paint_rect.left_center().y,
                },
                tab_intel_galley.size(),
            );

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

            //Starts drawing the rest of the folders and files
            let potential_new_root = self.follow_paint_order(ui, root_draw_anchor);
            //assigning a new root if selected
            match potential_new_root {
                Some(_) => self.change_root(potential_new_root.unwrap()),
                None => (),
            }
        });
    }
}
