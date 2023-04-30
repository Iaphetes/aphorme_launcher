#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
                                                                   // #![feature(map_try_insert)]
mod apps;
mod config;
mod egui_ui;
mod iced_ui;
use crate::apps::ApplicationManager;
use crate::config::{load_config, Config};
#[cfg(feature = "egui-ui")]
use crate::egui_ui::egui_ui::launch_egui_ui;
#[cfg(feature = "iced-ui")]
use crate::iced_ui::iced_ui::launch_iced_ui;
use config::UIFramework;
use single_instance::SingleInstance;
fn main() {
    let instance = SingleInstance::new("Aphorme").unwrap();
    if instance.is_single() {
        let cfg: Config = load_config(None);
        let application_manager: ApplicationManager =
            ApplicationManager::new(cfg.app_cfg.unwrap_or_default(), cfg.gui_cfg.icon, instance);
        let gui_framework: UIFramework = cfg.gui_cfg.ui_framework.unwrap_or_default();
        // let gui_framework: GuiFramework = GuiFramework::EGUI; //cfg.ui_framework.unwrap_or_default();
        match gui_framework {
            UIFramework::EGUI => {
                #[cfg(feature = "egui-ui")]
                match launch_egui_ui(cfg.gui_cfg, application_manager) {
                    Ok(()) => {}
                    Err(error) => println!("{error:?}"),
                };
                #[cfg(not(feature = "egui-ui"))]
                panic!("Trying to use egui without \"ui-egui\"-feature activated");
            }

            UIFramework::ICED => {
                #[cfg(feature = "iced-ui")]
                launch_iced_ui(cfg.gui_cfg, application_manager);
            }
        }
    } else {
        println!("another instance is already running");
    }
}
