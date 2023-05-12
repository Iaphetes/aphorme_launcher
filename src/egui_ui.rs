#[cfg(feature = "egui-ui")]
pub mod egui_ui {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::apps::{Application, ApplicationManager};
    use crate::config::GuiCFG;
    use eframe::{
        egui::{self, Key, RichText},
        epaint::{Color32, TextureId, Vec2},
    };
    use egui_extras::RetainedImage;
    use log::debug;
    pub fn launch_egui_ui(
        gui_cfg: GuiCFG,
        application_manager: ApplicationManager,
    ) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(320.0, 240.0)),
            always_on_top: true,
            decorated: false,
            centered: true,
            resizable: false,
            ..Default::default()
        };

        eframe::run_native(
            "Aphorme",
            options,
            Box::new(move |_cc| Box::new(EguiUI::new(gui_cfg.clone(), application_manager))),
        )
    }
    struct EguiUI {
        /// Selected element in list of applications
        selected: usize,
        /// Struct managing searching for, loading icons of, matching and running applications
        application_manager: ApplicationManager,
        /// The user entered search string
        search_str: String,
        /// Map containing the Optional TextureIds of the icons matched to the corresponding
        /// application name
        icon_ids: HashMap<String, Option<TextureId>>,
        /// The application icons
        icons: Vec<RetainedImage>,
        /// Empty icon to display if no icon has been found yet for the application
        placeholder_icon: Option<TextureId>,
        /// The GUI configuration
        gui_cfg: GuiCFG,
    }

    impl EguiUI {
        pub fn new(gui_cfg: GuiCFG, application_manager: ApplicationManager) -> Self {
            Self {
                selected: 0,
                application_manager,
                search_str: "".to_string(),
                icon_ids: HashMap::new(),
                icons: Vec::new(),
                placeholder_icon: None,
                gui_cfg,
            }
        }
        /// Custom scrolling function using the arrow keys or the scroll delta of the mouse wheel.
        /// Always keeps the selected item on top
        fn scroll(&mut self, ctx: &egui::Context) {
            let down: bool = ctx.input(|i| {
                i.key_pressed(Key::ArrowDown)
                    || i.key_pressed(Key::ArrowRight)
                    || i.scroll_delta.y < -1.0
            });
            let up: bool = ctx.input(|i| {
                i.key_pressed(Key::ArrowUp)
                    || i.key_pressed(Key::ArrowLeft)
                    || i.scroll_delta.y > 1.0
            });
            if down && self.selected < self.application_manager.matches.len() - 1 {
                self.selected += 1;
            }
            if up && self.selected > 0 {
                self.selected -= 1;
            }
        }
        /// Get the icon from the HashMap. If it is not found load it in and add it to the HashMap.
        /// Returns None if no icon can be found
        fn get_icon(
            &mut self,
            application: &Application,
            ctx: &egui::Context,
        ) -> Option<TextureId> {
            match self.icon_ids.get(&application.name) {
                Some(handle) => *handle,
                None => {
                    let icon_path: &PathBuf = &application.icon_path.clone()?;
                    let ext: &str = icon_path.extension()?.to_str()?;

                    let data: Vec<u8> = std::fs::read(icon_path).ok()?;
                    let image_res: Option<RetainedImage> = match ext {
                        "png" | "jpg" | "jpeg" => {
                            egui_extras::image::RetainedImage::from_image_bytes(
                                &application.name,
                                data.as_slice(),
                            )
                            .ok()
                        }
                        "svg" => egui_extras::image::RetainedImage::from_svg_bytes(
                            &application.name,
                            data.as_slice(),
                        )
                        .ok(),
                        _ => {
                            debug!("Unknown file extension {:?}", ext);
                            None
                        }
                    };
                    let image: RetainedImage = image_res?;
                    let id: TextureId = image.texture_id(&ctx);
                    if !self.icon_ids.contains_key(&application.name) {
                        self.icon_ids.insert(application.name.clone(), Some(id));
                    }
                    self.icons.push(image);
                    Some(id)
                }
            }
        }
    }

    impl eframe::App for EguiUI {
        fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
            egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
        }
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            self.scroll(ctx);
            let execute: bool = ctx.input(|i| i.key_pressed(Key::Enter));
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                _frame.close();
            }
            if execute {
                self.application_manager.execute_first_match(self.selected);
                _frame.close();
            }
            if self.gui_cfg.icon {
                self.application_manager.load_next_icons(5);
            }
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = ui.add(egui::TextEdit::singleline(&mut self.search_str));
                response.request_focus();
                if response.changed() {
                    self.application_manager.find_application(&self.search_str);
                    self.selected = 0;
                }
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_width(f32::INFINITY)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for (i, (application, _)) in (self.application_manager.matches.clone())
                            .iter()
                            .enumerate()
                        {
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
                            let response = egui::Frame::none()
                                .fill(background_color)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if self.gui_cfg.icon {
                                            ui.image(icon, Vec2 { x: 8.0, y: 8.0 });
                                        }
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
            ctx.request_repaint();
        }
    }
}
