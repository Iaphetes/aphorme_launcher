use std::collections::HashMap;

use crate::apps::{collect_applications, find_application, Application};
use crate::config::GuiCFG;
use eframe::{
    egui::{self, Key, RichText},
    epaint::{Color32, TextureId, Vec2},
};
use egui_extras::RetainedImage;
pub fn launch_egui_ui(gui_cfg: &GuiCFG) -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    // tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        always_on_top: true,
        decorated: false,
        centered: true,
        resizable: false,
        // fullscreen: true,
        ..Default::default()
    };
    let mut applications: Vec<Application> = collect_applications(gui_cfg.icon);
    applications.sort();
    let app: EguiUI = EguiUI {
        selected: 0,
        applications: applications.clone(),
        matches: applications.into_iter().map(|a| (a, 0)).collect(),
        search_str: "".to_string(),
        icon_ids: HashMap::new(),
        icons: Vec::new(),
        placeholder_icon: None,
    };

    eframe::run_native("Aphorme", options, Box::new(|_cc| Box::new(app)))
}
struct EguiUI {
    /// Selected element in list of applications
    selected: usize,
    /// List of applications on the system
    applications: Vec<Application>,
    /// Matches found with the `search_str` with the corresponding match score
    matches: Vec<(Application, i64)>,
    /// The user entered search string
    search_str: String,
    /// Map containing the Optional TextureIds of the icons matched to the corresponding
    /// application name
    icon_ids: HashMap<String, Option<TextureId>>,
    /// The application icons
    icons: Vec<RetainedImage>,
    /// Empty icon to display if no icon has been found yet for the application
    placeholder_icon: Option<TextureId>,
}

impl Default for EguiUI {
    fn default() -> Self {
        let mut applications: Vec<Application> = collect_applications(true);
        applications.sort();
        Self {
            selected: 0,
            applications: applications.clone(),
            matches: applications.into_iter().map(|a| (a, 0)).collect(),
            search_str: "".to_string(),
            icon_ids: HashMap::new(),
            icons: Vec::new(),
            placeholder_icon: None,
        }
    }
}
impl EguiUI {
    /// Custom scrolling function using the arrow keys or the scroll delta of the mouse wheel.
    /// Always keeps the selected item on top
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
    /// Get the icon from the HashMap. If it is not found load it in and add it to the HashMap.
    /// Returns None if no icon can be found
    fn get_icon(&mut self, application: &Application, ctx: &egui::Context) -> Option<TextureId> {
        match self.icon_ids.get(&application.name) {
            Some(handle) => *handle,
            None => {
                match &application.icon_path {
                    Some(icon_path) => {
                        match icon_path.extension() {
                            Some(ext) => match ext.to_str() {
                                Some("png") | Some("jpg") | Some("jpeg") | Some("svg") => {
                                    match std::fs::read(icon_path) {
                                        Ok(data) => {
                                            let image_res: Result<RetainedImage, String>;
                                            if icon_path.to_string_lossy().ends_with("svg") {
                                                image_res = egui_extras::image::RetainedImage::from_svg_bytes(&application.name, data.as_slice());
                                            } else {
                                                image_res = egui_extras::image::RetainedImage::from_image_bytes(&application.name, data.as_slice());
                                            }
                                            match image_res {
                                                Ok(image) => {
                                                    let id: TextureId = image.texture_id(&ctx);
                                                    self.icons.push(image);
                                                    Some(id)
                                                }
                                                Err(error) => {
                                                    println!("Error while reading icon {}", error);
                                                    None
                                                }
                                            }
                                        }
                                        Err(error) => {
                                            println!("Error while parsing svg file {:?}", error);
                                            None
                                        }
                                    }
                                }
                                _ => {
                                    println!("Unknown file extension {:?}", ext);
                                    None
                                }
                            },
                            None => None,
                        }
                    }
                    None => None,
                }
            }
        }
    }
}

impl eframe::App for EguiUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.scroll(&ctx);
        let execute: bool = ctx.input(|i| i.key_pressed(Key::Enter));
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            std::process::exit(0);
        }
        if execute {
            self.matches[self.selected].0.run(true);
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
                    for (i, (application, _)) in (&self.matches.clone()).into_iter().enumerate() {
                        let label_text: RichText = RichText::new(application.name.clone());
                        let mut background_color: Color32 =
                            Color32::from_rgba_unmultiplied(0, 0, 0, 0);
                        if i == self.selected {
                            background_color = Color32::from_rgba_unmultiplied(0, 100, 0, 128);
                        }
                        let icon_texture_handle: Option<TextureId> =
                            self.get_icon(application, ctx);
                        let icon: TextureId = match icon_texture_handle {
                            Some(handle) => handle,
                            None => match self.placeholder_icon {
                                Some(icon_handle) => icon_handle,
                                None => {
                                    let icon_handle = ctx
                                        .load_texture(
                                            "placeholder",
                                            egui::ColorImage::new(
                                                [8, 8],
                                                Color32::from_rgba_unmultiplied(0, 0, 0, 0),
                                            ),
                                            Default::default(),
                                        )
                                        .id();
                                    self.placeholder_icon = Some(icon_handle);
                                    icon_handle
                                }
                            },
                        };
                        let _ = self
                            .icon_ids
                            .try_insert(application.name.clone(), Some(icon.clone()));

                        let response = egui::Frame::none()
                            .fill(background_color)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.image(icon, Vec2 { x: 8.0, y: 8.0 });
                                    ui.label(label_text);
                                })
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
