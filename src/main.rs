#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![feature(map_try_insert)]
mod apps;
mod config;
mod egui_ui;
use crate::apps::ApplicationManager;
use crate::config::{load_config, Config};
use single_instance::SingleInstance;
fn main() {
    let instance = SingleInstance::new("Aphorme").unwrap();
    if instance.is_single() {
        let cfg: Config = load_config(None);
        let application_manager: ApplicationManager =
            ApplicationManager::new(cfg.app_cfg.unwrap_or_default(), cfg.gui_cfg.icon);
        match egui_ui::launch_egui_ui(cfg.gui_cfg, application_manager) {
            Ok(()) => {}
            Err(error) => println!("{error:?}"),
        };
    } else {
        println!("another instance is already running");
    }
}
