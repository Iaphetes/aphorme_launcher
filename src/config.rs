use confy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub gui_cfg: GuiCFG,
    pub app_cfg: Option<AppCFG>,
}
#[derive(Serialize, Deserialize, Debug)]
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
