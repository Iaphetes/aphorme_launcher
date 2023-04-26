use confy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Serialize, Deserialize, Debug)]
pub enum GuiFramework {
    EGUI,
    ICED,
}
impl Default for GuiFramework {
    fn default() -> Self {
        if cfg!(feature = "egui-ui") {
            return Self::EGUI;
        }
        if cfg!(feature = "iced-ui") {
            Self::ICED
        } else {
            Self::EGUI
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub gui_cfg: GuiCFG,
    pub app_cfg: Option<AppCFG>,
    pub ui_framework: Option<GuiFramework>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            gui_cfg: GuiCFG::default(),
            app_cfg: None,
            ui_framework: Some(GuiFramework::EGUI),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuiCFG {
    pub icon: bool,
}
impl Default for GuiCFG {
    fn default() -> Self {
        GuiCFG { icon: true }
    }
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppCFG {}
pub fn load_config(path: Option<PathBuf>) -> Config {
    match path {
        Some(p) => confy::load_path(p).expect("Configuration could not be loaded"),
        None => confy::load("aphorme", Some("config")).expect("Configuration could not be loaded"),
    }
}
