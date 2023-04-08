#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![feature(map_try_insert)]
mod apps;
use std::{collections::HashMap, ffi::OsString};

use apps::{collect_applications, Application};
use eframe::{
    egui::{self, Key, Label, Margin, RichText},
    epaint::{Color32, Rounding, TextureHandle, TextureId, Vec2},
};
use egui_extras::RetainedImage;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    // tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        always_on_top: true,
        decorated: false,
        centered: true,
        // fullscreen: true,
        ..Default::default()
    };
    let app: MyApp = MyApp::default();

    eframe::run_native("Aphorme", options, Box::new(|_cc| Box::new(app)))
}

struct MyApp {
    selected: usize,
    applications: Vec<Application>,
    matches: Vec<Application>,
    search_str: String,
    icon_ids: HashMap<String, Option<TextureId>>,
    icons: Vec<RetainedImage>,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut applications: Vec<Application> = collect_applications();
        applications.sort();

        Self {
            selected: 0,
            applications: applications.clone(),
            matches: applications,
            search_str: "".to_string(),
            icon_ids: HashMap::new(),
            icons: Vec::new(),
        }
    }
}
impl MyApp {
    fn scroll(&mut self, ctx: &egui::Context) {
        let down: bool = ctx.input(|i| i.key_pressed(Key::ArrowDown) || i.scroll_delta.y < -1.0);
        let up: bool = ctx.input(|i| i.key_pressed(Key::ArrowUp) || i.scroll_delta.y > 1.0);
        if down && self.selected < 100 {
            self.selected += 1;
        }
        if up && self.selected > 0 {
            self.selected -= 1;
        }
    }
}
fn find_application(
    search_str: &str,
    applications: &Vec<Application>,
    matches: &mut Vec<Application>,
) {
    let matcher = SkimMatcherV2::default();
    matches.clear();
    for application in applications {
        let search_match: Option<i64> = matcher.fuzzy_match(&application.name, search_str);
        println!(
            "{} = {} : {:?}",
            search_str, &application.name, search_match
        );
        if search_match.is_some() {
            matches.push(application.clone());
        }
    }
}
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.scroll(&ctx);
        let execute: bool = ctx.input(|i| i.key_pressed(Key::Enter));
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            std::process::exit(0);
        }
        if execute {
            self.matches[self.selected].run(true);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let response = ui.add(egui::TextEdit::singleline(&mut self.search_str));
            response.request_focus();
            if response.changed() {
                find_application(&self.search_str, &self.applications, &mut self.matches);
                self.selected = 0;
            }
            ui.separator();

            egui::ScrollArea::vertical()
                .max_width(f32::INFINITY)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (i, application) in (&self.matches).into_iter().enumerate() {
                        let label_text: RichText = RichText::new(application.name.clone());
                        let mut background_color: Color32 =
                            Color32::from_rgba_unmultiplied(0, 0, 0, 0);
                        if i == self.selected {
                            background_color = Color32::from_rgba_unmultiplied(0, 100, 0, 128);
                        }
                        let icon_texture_handle: Option<TextureId> =
                            match self.icon_ids.get(&application.name) {
                                Some(handle) => *handle,
                                None => match &application.icon_path {
                                    Some(icon_path) => match icon_path.extension() {
                                        Some(ext) => match ext.to_str() {
                                            Some("svg") => match std::fs::read(icon_path) {
                                                Ok(data) => match egui_extras::image::RetainedImage::from_svg_bytes(&application.name, data.as_slice()){
                                                Ok(image) => {
                                                    let id : TextureId = image.texture_id(&ctx);
                                                    self.icons.push(image);
                                                    Some(id)}
                                                ,
                                                Err(error) => {println!("Error while reading icon {}", error); None}
                                            },Err(error) => {
                                                println!("Error while parsing svg file {:?}", error);
                                                None}
                                            },
                                            Some("png") | Some("jpg") | Some("jpeg") => {match std::fs::read(icon_path) {
                                                Ok(data) => match egui_extras::image::RetainedImage::from_image_bytes(&application.name, data.as_slice()){
                                                Ok(image) => {
                                                    let id : TextureId = image.texture_id(&ctx);
                                                    self.icons.push(image);
                                                    Some(id)},
                                                Err(error) => {println!("Error while reading icon {}", error); None}
                                            },Err(error) => {
                                                println!("Error while parsing svg file {:?}", error);
                                                None}
                                            }}
                                            _ => {
                                                println!("Unknown file extension {:?}", ext);
                                                None
                                            }
                                        },
                                        None => None,
                                    },
                                    None => None,
                                }
                            };
                        let icon: TextureId =
                        match icon_texture_handle{
                            Some(handle) => handle,
                            None => ctx.load_texture(&application.name, egui::ColorImage::new([8, 8], Color32::from_rgba_unmultiplied(0, 0, 0, 0)), Default::default()).id()
                        };
                        self.icon_ids.try_insert(application.name.clone(), Some(icon.clone()));
                        let response = egui::Frame::none()
                            .fill(background_color)
                            .show(ui, |ui| {
                                ui.image(icon, Vec2{x: 8.0, y: 8.0});
                                ui.label(label_text);
                            })
                            .response;
                        if i == self.selected {
                            response.scroll_to_me(Some(egui::Align::Min));
                        }
                        // std::process::exit(0);
                    }
                });
        });
    }
}
