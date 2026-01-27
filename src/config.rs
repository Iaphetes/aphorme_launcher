use log::error;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub gui_cfg: GuiCFG,
    pub app_cfg: Option<AppCFG>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuiCFG {
    pub icon: bool,
    pub retain_focus: bool,
    pub font_size: u8,
    pub window_size: (u32, u32),
}
impl Default for GuiCFG {
    fn default() -> Self {
        GuiCFG {
            icon: true,
            retain_focus: true,
            font_size: 12,
            window_size: (320, 240),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrefCFG {
    pub max_weight: i64,
}

impl Default for PrefCFG {
    fn default() -> Self {
        PrefCFG { max_weight: 10 }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppCFG {
    pub paths: Option<Vec<String>>,
    pub use_default_paths: Option<bool>,
    pub preferred_apps: Option<PrefCFG>,
}
impl Default for AppCFG {
    fn default() -> Self {
        AppCFG {
            paths: Some(Vec::new()),
            use_default_paths: Some(true),
            preferred_apps: Some(PrefCFG { max_weight: 10 }),
        }
    }
}
pub fn load_config(path: Option<PathBuf>) -> Config {
    match path {
        Some(p) => confy::load_path(p).expect("Configuration could not be loaded"),
        None => {
            println!("loading configconfig");

            match confy::load("aphorme", Some("config")) {
                Ok(config) => config,
                Err(err) => {
                    error!("{:?}", err);
                    let config: Config = Config::default();
                    let _ = confy::store("aphorme", Some("config"), Config::default());
                    config
                }
            }
            // .ok()
            // .unwrap_or_else(|| {
            // })
        }
    }
}
