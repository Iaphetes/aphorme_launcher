#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![feature(map_try_insert)]
mod apps;
mod config;
mod egui_ui;
use crate::config::{load_config, Config, GuiCFG};
fn main() {
    // let mut config_path: PathBuf = home_dir().expect("No home directory ");
    // config_path.push(".config/aphorme/config.toml");
    let cfg: Config = load_config(None);
    println!("{:#?}", cfg);
    match egui_ui::launch_egui_ui(&cfg.gui_cfg) {
        Ok(()) => {}
        Err(error) => println!("{error:?}"),
    };
}
