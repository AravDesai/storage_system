use std::fs;
use lb_rs::{Config, Core, File};
use serde::{Deserialize, Serialize};
use eframe::egui::{self, Color32, Context, Pos2, Rect};


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
        Box::new(|cc| Box::new(MyApp::init(cc.egui_ctx.clone()))),
    );
}

struct MyApp{
    current_root: Data,
}

impl MyApp{
    fn init(ctx: Context) -> Self {
        let json_info = fs::read_to_string("parth-doc-data.json").expect("Couldn't read file");
        let data: Vec<Data> = serde_json::from_str(&json_info).expect("Json not formatted well");
        let mut root= data[0].clone();
        for item in data{
            if item.file.id == item.file.parent{
                root = item;
            }
        }
        Self{
            current_root: root,
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
                painter.circle(
                    window_size.max/2.0,
                    50.0,
                    Color32::WHITE,
                    egui::Stroke {
                        width: 0.0,
                        color: Color32::from_rgb(255, 255, 255),
                    },
                );
                ui.allocate_ui_at_rect(center_text, |ui|{
                    ui.label(self.current_root.size.to_string() + " MB").on_hover_text(self.current_root.file.name.to_string());
                });
            });
        });
    }
}