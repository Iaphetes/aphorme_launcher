#[cfg(feature = "egui-ui")]
pub mod ui {
    use std::{fs::File, io::Read};

    use crate::apps::ApplicationManager;
    use crate::config::GuiCFG;
    use eframe::egui::TextBuffer;
    use eframe::{
        egui::{self, FontId, Image, Key, RichText, ViewportCommand},
        epaint::{Color32, Vec2},
    };

    use log::debug;
    pub fn launch_egui_ui(
        gui_cfg: GuiCFG,
        application_manager: ApplicationManager,
    ) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([gui_cfg.window_size.0 as f32, gui_cfg.window_size.1 as f32])
                .with_decorations(false)
                .with_resizable(false)
                .with_always_on_top(),
            centered: true,
            ..Default::default()
        };

        eframe::run_native(
            "Aphorme",
            options,
            Box::new(move |cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);

                Box::new(EguiUI::new(gui_cfg.clone(), application_manager))
            }),
        )
    }
    struct EguiUI {
        /// Selected element in list of applications
        selected: usize,
        /// Struct managing searching for, loading icons of, matching and running applications
        application_manager: ApplicationManager,
        /// The user entered search string
        search_str: String,
        /// The GUI configuration
        gui_cfg: GuiCFG,
    }

    impl EguiUI {
        pub fn new(gui_cfg: GuiCFG, application_manager: ApplicationManager) -> Self {
            Self {
                selected: 0,
                application_manager,
                search_str: "".to_string(),
                gui_cfg,
            }
        }
        /// Custom scrolling function using the arrow keys or the scroll delta of the mouse wheel.
        /// Always keeps the selected item on top
        fn scroll(&mut self, ctx: &egui::Context) {
            let down: bool = ctx.input(|i| {
                i.key_pressed(Key::ArrowDown)
                    || i.key_pressed(Key::ArrowRight)
                    || i.key_pressed(Key::Tab)
                    || i.raw_scroll_delta.y < -1.0
            });
            let up: bool = ctx.input(|i| {
                i.key_pressed(Key::ArrowUp)
                    || i.key_pressed(Key::ArrowLeft)
                    || i.raw_scroll_delta.y > 1.0
            });
            if down && self.selected < self.application_manager.matches.len() - 1 {
                self.selected += 1;
            }
            if up && self.selected > 0 {
                self.selected -= 1;
            }
        }
    }

    impl eframe::App for EguiUI {
        fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
            egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
        }
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            self.scroll(ctx);
            if self.gui_cfg.retain_focus {
                ctx.send_viewport_cmd(ViewportCommand::Focus)
            }
            let execute: bool = ctx.input(|i| i.key_pressed(Key::Enter));
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                ctx.send_viewport_cmd(ViewportCommand::Close)
            }
            if execute {
                self.application_manager.execute_first_match(self.selected);
                ctx.send_viewport_cmd(ViewportCommand::Close)
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
                            let label_text: RichText = RichText::new(application.name.clone())
                                .font(FontId::proportional(self.gui_cfg.font_size as f32));
                            let mut background_color: Color32 =
                                Color32::from_rgba_unmultiplied(0, 0, 0, 0);
                            if i == self.selected {
                                background_color = Color32::from_rgba_unmultiplied(0, 100, 0, 128);
                            }
                            let icon_path = &application.icon_path.clone();
                            let image: Option<Image> = match icon_path {
                                Some(icon_path) => {
                                    if let Some(ext_raw) = icon_path.extension() {
                                        match ext_raw.to_string_lossy().as_str() {
                                            "png" | "jpg" | "jpeg" | "svg" => {
                                                let file_uri =
                                                    icon_path.to_string_lossy().into_owned();
                                                // println!("{}", file_uri);
                                                let mut file = File::open(&file_uri.clone())
                                                    .expect("Failed to open image file");
                                                let mut bytes = Vec::new();
                                                file.read_to_end(&mut bytes)
                                                    .expect("Failed to read image file");

                                                Some(egui::Image::from_bytes(
                                                    format!("bytes://{}", file_uri),
                                                    bytes,
                                                ))
                                            }
                                            _ => {
                                                debug!("Unknown file extension {:?}", ext_raw);
                                                None
                                            }
                                        }
                                    } else {
                                        None
                                    }
                                }
                                None => None,
                            };

                            // let icon: TextureId = match icon_texture_handle {
                            //     Some(handle) => handle,
                            //     None => match self.placeholder_icon {
                            //         Some(icon_handle) => icon_handle,
                            //         None => {
                            //             let icon_handle = ctx
                            //                 .load_texture(
                            //                     "placeholder",
                            //                     egui::ColorImage::new(
                            //                         [8, 8],
                            //                         Color32::from_rgba_unmultiplied(0, 0, 0, 0),
                            //                     ),
                            //                     Default::default(),
                            //                 )
                            //                 .id();
                            //             self.placeholder_icon = Some(icon_handle);
                            //             icon_handle
                            //         }
                            //     },
                            // };
                            let response = egui::Frame::none()
                                .fill(background_color)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if let Some(actual_image) = image {
                                            ui.add(
                                                actual_image
                                                    .max_size(Vec2::new(
                                                        self.gui_cfg.font_size as f32,
                                                        self.gui_cfg.font_size as f32,
                                                    ))
                                                    .show_loading_spinner(true),
                                            );
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
