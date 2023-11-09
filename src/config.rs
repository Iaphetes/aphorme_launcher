use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum UIFramework {
    Egui,
    Iced,
}
impl Default for UIFramework {
    fn default() -> Self {
        if cfg!(feature = "egui-ui") {
            return Self::Egui;
        }
        if cfg!(feature = "iced-ui") {
            Self::Iced
        } else {
            Self::Egui
        }
    }
}
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub gui_cfg: GuiCFG,
    pub app_cfg: Option<AppCFG>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuiCFG {
    pub icon: bool,
    pub ui_framework: Option<UIFramework>,
    pub retain_focus: bool,
}
impl Default for GuiCFG {
    fn default() -> Self {
        GuiCFG {
            icon: true,
            ui_framework: None,
            retain_focus: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrefCFG {
    pub max_weight: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppCFG {
    pub paths: Vec<String>,
    pub use_default_paths: Option<bool>,
    pub preferred_apps: PrefCFG,
}
impl Default for AppCFG {
    fn default() -> Self {
        AppCFG {
            paths: Vec::new(),
            use_default_paths: Some(true),
            preferred_apps: PrefCFG { max_weight: 10 },
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
