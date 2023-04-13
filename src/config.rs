use confy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub gui_cfg: GuiCFG,
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
pub fn load_config(path: Option<PathBuf>) -> Config {
    match path {
        Some(p) => confy::load_path(p).expect("Configuration could not be loaded"),
        None => confy::load("aphorme", Some("config")).expect("Configuration could not be loaded"),
    }
}
