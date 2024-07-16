use std::fs;
use lb_rs::{Config, Core, File, Uuid};
use serde::{Deserialize, Serialize};
use eframe::egui::{self, Align2, Color32, Context, FontFamily, FontId, Id, LayerId, Pos2, Rect};
use egui_circle_trim::egui_circle_trim::CircleTrim;

pub mod egui_circle_trim;

#[derive(Debug, Deserialize, Clone)]
struct Data {
    file: File,
    size: u64,
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

struct MyApp{
    current_root: Data,
    inner_radius: f32,
    outer_radius: f32,
    start_angle: i32,
    data: Vec<Data>,
}

impl MyApp{
    fn init(ctx: Context) -> Self {
        let json_info = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        let data: Vec<Data> = serde_json::from_str(&json_info).expect("Json not formatted well");
        let data_clone = data.clone();
        let mut root= data[0].clone();
        for item in data{
            if item.file.id == item.file.parent{
                root = item;
            }
        }
        Self{
            data: data_clone,
            current_root: root,
            inner_radius: 50.0,
            outer_radius: 70.0,
            start_angle: 0,
        }
    }
}

fn file_recurser(parent: Data, data: Vec<Data>, root: Data){
    let mut files = vec![];
    for file in data{
        if(file.file.parent == parent.file.id && file.file.id != root.file.id){
            files.push(file);
        }
    }

}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let window_size = ctx.input(|i: &egui::InputState| i.screen_rect());

            ui.heading("Storage Viewer");

            let center = Rect{
                min: window_size.max/2.0,
                max: window_size.max,
            };

            let center_text= Rect{
                min: Pos2{
                    x: (window_size.max.x/2.0) - 25.0,
                    y: (window_size.max.y/2.0) - 5.0,
                },
                max: window_size.max,
            };
            ui.allocate_ui_at_rect(center, |ui|{
                let painter = ui.painter();
                painter.clone().with_layer_id(LayerId{ order: egui::Order::PanelResizeLine, id: Id::new(2)}).circle(
                    window_size.max/2.0,
                    50.0,
                    Color32::WHITE,
                    egui::Stroke {
                        width: 0.0,
                        color: Color32::from_rgb(255, 255, 255),
                    },
                );
                // painter.clone().with_layer_id(LayerId{ order: egui::Order::PanelResizeLine, id: Id::new(4)}).circle(
                //     window_size.max/2.0,
                //     80.0,
                //     Color32::BLUE,
                //     egui::Stroke {
                //         width: 0.0,
                //         color: Color32::from_rgb(255, 255, 255),
                //     },
                // );
                let trim = CircleTrim::new(Color32::BLUE, self.inner_radius, ctx.clone(), LayerId{ order: egui::Order::PanelResizeLine, id: Id::new(2)}, Id::new(2), center, ui.painter().clone(), 0, 90, center.min);

                ui.add(trim);
                //egui_circle_trim::egui_circle_trim::CircleTrim::paint_annulus_sector(painter, center.min, 50.0, self.start_angle, 90, Color32::BLUE);
                    painter.clone().with_layer_id(LayerId{ order: egui::Order::Foreground, id: Id::new(1)}).text(center.min, Align2::CENTER_CENTER, self.current_root.size.to_string() + " MB", FontId{ size: 15.0, family: FontFamily::Proportional }, Color32::BLACK);
                    ui.allocate_ui_at_rect(center_text, |ui|{
                        ui.label("000000").on_hover_text(self.current_root.file.name.to_string());
                    });
                file_recurser(self.current_root.clone(), self.data.clone(), self.current_root.clone());
            });
        });
    }
}