#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![feature(map_try_insert)]
mod apps;
mod egui_ui;
fn main() {
    match egui_ui::launch_egui_ui() {
        Ok(()) => {}
        Err(error) => println!("{error:?}"),
    };
}
