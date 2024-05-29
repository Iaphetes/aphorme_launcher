use crate::config::{AppCFG, PrefCFG};
use freedesktop_entry_parser::{parse_entry, Entry};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use linicon::lookup_icon;
use linicon_theme::get_icon_theme;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use single_instance::SingleInstance;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};
/// The paths where the desktop files and binaries are located. Will be exported to a config file
/// and inserted in the defaults
const APPLICATION_PATHS: [&str; 6] = [
    "/usr/share/applications",
    "/usr/local/share/applications",
    "$HOME/.local/share/applications",
    "/var/lib/flatpak/exports/share/applications",
    "/run/current-system/sw/share/applications",
    "$HOME/.local/state/home-manager/gcroots/current-home/home-path/share/applications"
];
/// The type of application. Either a binary (not yet supported) or a Desktop file
#[derive(Clone, Eq, PartialEq, Default, Serialize, Deserialize, Hash, Debug)]
enum ApplicationType {
    #[default]
    DesktopFile,
    Stdout, // BINARY,
}
const LOCAL_DIR: &str = "$HOME/.local/share/aphorme/preferred_apps.json";
#[derive(Default, Serialize, Deserialize)]
struct PreferredApps {
    path: PathBuf,
    weight_map: HashMap<String, i64>,
    max_weight: i64,
}
impl PreferredApps {
    pub fn new(home_dir: &str, preference_cfg: &PrefCFG) -> Self {
        let mut preferred_map: PreferredApps = PreferredApps {
            path: PathBuf::from(LOCAL_DIR.replace("$HOME", home_dir)),
            weight_map: HashMap::new(),
            max_weight: preference_cfg.max_weight,
        };
        if Path::new(&preferred_map.path).exists() {
            if let Ok(preference_file_content) = fs::read_to_string(&preferred_map.path) {
                preferred_map.weight_map = serde_json::from_str(&preference_file_content)
                    .ok()
                    .unwrap_or_default();
            };
        }
        preferred_map
    }
    pub fn save(&self) {
        // error!("{:#?}", self.weight_map);
        if let Some(containing_folder) = self.path.parent() {
            if !Path::new(&containing_folder).exists() {
                if let Err(err) = std::fs::create_dir_all(containing_folder) {
                    error!("{:?}", err);
                    return;
                }
            }
            if let Ok(mut fileptr) = File::create(&self.path) {
                // error!("{:#?}", serde_json::to_string(&self.weight_map));
                if let Err(err) = fileptr.write_all(
                    serde_json::to_string(&self.weight_map)
                        .unwrap_or_default()
                        .as_bytes(),
                ) {
                    error!("{:?}", err);
                }
            }
        }
    }
    pub fn update_preferrence(&mut self, application: &Application) {
        match self.weight_map.get_mut(&application.name) {
            Some(weight) => {
                if *weight <= self.max_weight {
                    *weight += 1;
                }
            }
            None => {
                self.weight_map.insert(application.name.clone(), 1);
            }
        };
    }
    pub fn get_weight(&self, application: &Application) -> i64 {
        *self.weight_map.get(&application.name).unwrap_or(&0)
    }
}
#[derive(Default)]
pub struct ApplicationManager {
    applications: Vec<Application>,
    pub matches: Vec<(Application, i64)>,
    icon_theme: String,
    loaded_icons: usize,
    instance: Option<SingleInstance>,
    preferred_applications: PreferredApps,
}
impl ApplicationManager {
    pub fn new(
        config: AppCFG,
        icon: bool,
        instance: SingleInstance,
        custom_fields: Vec<String>,
    ) -> ApplicationManager {
        let mut paths: Vec<String> = config.paths.clone();
        if config.use_default_paths.is_none() || config.use_default_paths == Some(true) {
            for path in Vec::from(APPLICATION_PATHS)
                .into_iter()
                .map(|p| p.to_owned())
            {
                paths.push(path);
            }
        }

        let home_dir_opt = env::var_os("HOME");

        let mut preferred_apps: Option<PreferredApps> = None;
        match home_dir_opt {
            Some(home_dir) => {
                paths = paths
                    .into_iter()
                    .map(|p| p.replace("$HOME", &home_dir.to_string_lossy()))
                    .collect();
                preferred_apps = Some(PreferredApps::new(
                    &home_dir.to_string_lossy(),
                    &config.preferred_apps,
                ));
            }
            None => warn!("Impossible to get your home dir!"),
        };
        let mut applications: Vec<Application>;
        if custom_fields.is_empty() {
            applications = collect_applications(&paths);
            applications.sort();
        } else {
            applications = Vec::new();
            for field in custom_fields {
                // debug!("{}", field);
                applications.push(Application {
                    name: field.clone(),
                    command: field,
                    icon_path: None,
                    icon_name: None,
                    application_type: ApplicationType::Stdout,
                });
            }
        }

        ApplicationManager {
            applications: applications.clone(),
            matches: applications.into_iter().map(|app| (app, 0)).collect(),
            icon_theme: match icon {
                true => get_icon_theme().unwrap_or_else(|| {
                    warn!("No icon theme found");
                    "".to_string()
                }),
                false => String::new(),
            },
            loaded_icons: 0,
            instance: Some(instance),
            preferred_applications: preferred_apps.unwrap_or_default(),
        }
    }
    /// Clear the Matches and then from the vector of applications fuzzy find the search_str and  append to the matches
    pub fn find_application(&mut self, search_str: &str) {
        let matcher = SkimMatcherV2::default();
        self.matches.clear();
        for application in &self.applications {
            let search_match: Option<i64> = matcher.fuzzy_match(&application.name, search_str);
            debug!(
                "{} = {} : {:?}",
                search_str, &application.name, search_match
            );
            if let Some(score) = search_match {
                self.matches.push((
                    application.clone(),
                    score + self.preferred_applications.get_weight(application),
                ));
            }
        }
        self.matches.sort_by(|a, b| b.1.cmp(&a.1));
    }
    pub fn execute_first_match(&mut self, selected: usize) {
        self.instance = None;
        let selected_match: &Application = &self.matches[selected].0;

        match selected_match.application_type {
            ApplicationType::DesktopFile => {
                self.preferred_applications
                    .update_preferrence(selected_match);
                self.preferred_applications.save();
                selected_match.run(false);
            }
            #[allow(clippy::print_stdout)]
            ApplicationType::Stdout => println!("{}", selected_match.command),
        }
    }
    pub fn load_next_icons(&mut self, amount: usize) -> bool {
        let mut is_done: bool = false;
        if self.loaded_icons < self.applications.len() {
            let last: usize = if self.loaded_icons + amount >= self.applications.len() {
                self.applications.len() - 1
            } else {
                self.loaded_icons + amount
            };
            for i in self.loaded_icons..last {
                self.applications[i].icon_path = match &self.applications[i].icon_name {
                    Some(path) => match lookup_icon(path)
                        .from_theme(&self.icon_theme)
                        .with_size(8)
                        .next()
                    {
                        Some(icon_path) => match icon_path {
                            Ok(linicon_path) => Some(linicon_path.path.clone()),
                            Err(_) => None,
                        },
                        None => None,
                    },
                    None => None,
                };

                if let Some(m) = self
                    .matches
                    .iter_mut()
                    .find(|m| m.0.name == self.applications[i].name)
                {
                    m.0.icon_path = self.applications[i].icon_path.clone();
                };
            }
            self.loaded_icons += amount;
        } else {
            is_done = true;
        }
        is_done
    }
}

/// A specific application found on the system
#[derive(Clone, Eq, PartialEq, Default, Serialize, Deserialize, Hash, Debug)]
pub struct Application {
    /// Name of the application as stated in the desktop file or the name of the executable if
    /// Application Type is binary
    pub name: String,
    /// The command to execute. Either the entry 'Exec' in the Desktop file or path to executable
    command: String,
    /// Optional icon path, if defined in the desktop file and found in the system
    pub icon_path: Option<PathBuf>,
    pub icon_name: Option<String>,
    /// The type of application
    application_type: ApplicationType,
}
impl Ord for Application {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}
impl PartialOrd for Application {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Application {
    /// Executes the program and exits if quit is true
    pub fn run(&self, quit: bool) {
        let split_command: Vec<&str> = self.command.split(' ').collect();
        let mut args: Vec<&str> = Vec::new();
        for arg in split_command[1..].iter() {
            if !arg.is_empty() && !arg.starts_with('%') {
                args.push(arg.to_owned());
            }
        }
        Command::new(split_command[0].trim_matches('\"'))
            .args(args)
            .spawn()
            .unwrap();
        if quit {}
    }
}
// fn search_icons(name: &str) {}
/// Find applications in the APPLICATION_PATHS and return them as a `Vec<Application>`
pub fn collect_applications(paths: &Vec<String>) -> Vec<Application> {
    debug!("{:#?}", paths);
    let mut applications: Vec<Application> = Vec::new();

    for path in paths {
        match fs::read_dir(path) {
            Ok(files) => {
                let path_applications: Vec<Option<Application>> = files
                    .collect::<Vec<Result<fs::DirEntry, std::io::Error>>>()
                    .iter()
                    .map(|file_res| match file_res {
                        Ok(file) => {
                            if !file.file_name().to_string_lossy().ends_with(".desktop") {
                                return None;
                            }
                            let entry: Entry = match parse_entry(file.path()) {
                                Ok(entry) => entry,
                                Err(err) => {
                                    error!(
                                        "Desktop file {} not readable, due to {:?}",
                                        file.path().to_string_lossy(),
                                        err
                                    );
                                    return None;
                                }
                            };
                            if let Some(nodisplay) =
                                entry.section("Desktop Entry").attr("NoDisplay")
                            {
                                if nodisplay == "true" {
                                    return None;
                                }
                            }

                            let name: Option<&str> = entry.section("Desktop Entry").attr("Name");
                            let command: Option<&str> = entry.section("Desktop Entry").attr("Exec");
                            let icon_path: Option<PathBuf> = None;
                            let icon_name: Option<String> = entry
                                .section("Desktop Entry")
                                .attr("Icon")
                                .map(|icon| icon.to_owned());
                            match (name, command) {
                                (Some(name), Some(command)) => Some(Application {
                                    name: name.into(),
                                    command: command.into(),
                                    icon_path,
                                    icon_name,
                                    application_type: ApplicationType::DesktopFile,
                                }),
                                _ => {
                                    error!(
                                        "Incomplete desktop file {}",
                                        file.path().to_string_lossy()
                                    );
                                    None
                                }
                            }
                        }
                        Err(error) => {
                            error!("Error encountered while reading file {:?}", error);
                            None
                        }
                    })
                    .collect();
                for application in path_applications.iter().flatten() {
                    applications.push(application.clone());
                }
            }
            Err(error) => {
                warn!("Could not read {path:?} because of {error:?}")
            }
        }
    }
    applications
}
