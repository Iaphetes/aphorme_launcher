use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum UIFramework {
    EGUI,
    ICED,
}
impl Default for UIFramework {
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub gui_cfg: GuiCFG,
    pub app_cfg: Option<AppCFG>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            gui_cfg: GuiCFG::default(),
            app_cfg: None,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuiCFG {
    pub icon: bool,
    pub ui_framework: Option<UIFramework>,
}
impl Default for GuiCFG {
    fn default() -> Self {
        GuiCFG {
            icon: true,
            ui_framework: None,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppCFG {
    pub paths: Vec<String>,
    pub use_default_paths: Option<bool>,
}
impl Default for AppCFG {
    fn default() -> Self {
        AppCFG {
            paths: Vec::new(),
            use_default_paths: Some(true),
        }
    }
}
pub fn load_config(path: Option<PathBuf>) -> Config {
    match path {
        Some(p) => confy::load_path(p).expect("Configuration could not be loaded"),
        None => confy::load("aphorme", Some("config"))
            .ok()
            .unwrap_or_else(|| {
                let config: Config = Config::default();
                let _ = confy::store("aphorme", Some("config"), Config::default());
                config
            }),
    }
}
