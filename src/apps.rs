use crate::config::AppCFG;
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
const APPLICATION_PATHS: [&str; 4] = [
    "/usr/share/applications",
    "/usr/local/share/applications",
    "$HOME/.local/share/applications",
    "/var/lib/flatpak/exports/share/applications",
];
/// The type of application. Either a binary (not yet supported) or a Desktop file
#[derive(Clone, Eq, PartialEq, Default, Serialize, Deserialize, Hash)]
enum ApplicationType {
    #[default]
    DesktopFile,
    Stdout, // BINARY,
}
const LOCAL_DIR: &str = "$HOME/.local/share/aphorme";
#[derive(Default, Serialize, Deserialize)]
struct PreferredMap(HashMap<Application, i64>);

#[derive(Default)]
pub struct ApplicationManager {
    applications: Vec<Application>,
    pub matches: Vec<(Application, i64)>,
    icon_theme: String,
    loaded_icons: usize,
    instance: Option<SingleInstance>,
    preferred_applications: PreferredMap,
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
        let mut preferred_map: PreferredMap = PreferredMap::default();
        match home_dir_opt {
            Some(home_dir) => {
                paths = paths
                    .into_iter()
                    .map(|p| p.replace("$HOME", &home_dir.to_string_lossy()))
                    .collect();
                let local_path: String = LOCAL_DIR.replace("$HOME", &home_dir.to_string_lossy());
                if Path::new(&local_path).exists() {
                    let preference_path: String = local_path + "/preferred_apps.json";
                    if let Ok(preference_file_content) = fs::read_to_string(&preference_path) {
                        preferred_map = serde_json::from_str(&preference_file_content)
                            .ok()
                            .unwrap_or_default();
                    };
                }
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
            preferred_applications: preferred_map,
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
                self.matches.push((application.clone(), score));
            }
        }
        self.matches.sort_by(|a, b| b.1.cmp(&a.1));
    }
    pub fn execute_first_match(&mut self, selected: usize) {
        self.instance = None;
        let selected_match: &Application = &self.matches[selected].0;

        match selected_match.application_type {
            ApplicationType::DesktopFile => {
                let home_dir_opt = env::var_os("$HOME");
                if let Some(home_dir) = home_dir_opt {
                    let local_path: String =
                        LOCAL_DIR.replace("$HOME", &home_dir.to_string_lossy());
                    if Path::new(&local_path).exists() {
                    } else {
                        //If let err
                        std::fs::create_dir_all(&local_path);
                    }
                    let preference_path: String = local_path + "/preferred_apps.json";
                    if let Ok(mut fileptr) = File::create(preference_path) {
                        if let Err(err) = fileptr.write_all(
                            serde_json::to_string(&self.preferred_applications)
                                .unwrap_or_default()
                                .as_bytes(),
                        ) {
                            error!("{:?}", err);
                        }
                    }
                };
                selected_match.run(false);
            }
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
#[derive(Clone, Eq, PartialEq, Default, Serialize, Deserialize, Hash)]
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
                    .map(|file_res| {
                        match file_res {
                            Ok(file) => {
                                if file.file_name().to_string_lossy().ends_with(".desktop") {
                                    let entry: Entry = parse_entry(file.path()).expect(&format!(
                                        "Desktop file {} not readable",
                                        file.path().to_string_lossy()
                                    ));
                                    let mut display: bool = true;
                                    match entry.section("Desktop Entry").attr("NoDisplay") {
                                        Some(nodisplay) => {
                                            if nodisplay == "true" {
                                                // continue;
                                                display = false;
                                            }
                                        }
                                        None => {}
                                    }
                                    if display {
                                        let name: &str = entry
                                            .section("Desktop Entry")
                                            .attr("Name")
                                            .expect(&format!(
                                                "Incomplete Desktop file {}",
                                                file.path().to_string_lossy()
                                            ));
                                        let command: &str = entry
                                            .section("Desktop Entry")
                                            .attr("Exec")
                                            .expect(&format!(
                                                "Incomplete Desktop file {}",
                                                file.path().to_string_lossy()
                                            ));
                                        let icon_path: Option<PathBuf> = None;
                                        let icon_name: Option<String> =
                                            match entry.section("Desktop Entry").attr("Icon") {
                                                Some(icon) => Some(icon.to_owned()),
                                                None => None,
                                            };
                                        Some(Application {
                                            name: name.into(),
                                            command: command.into(),
                                            icon_path,
                                            icon_name,
                                            application_type: ApplicationType::DesktopFile,
                                        })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            Err(error) => {
                                error!("Error encountered while reading file {:?}", error);
                                None
                            }
                        }
                    })
                    .collect();
                for application_opt in path_applications {
                    match application_opt {
                        Some(application) => applications.push(application),
                        None => {}
                    }
                }
            }
            Err(error) => {
                error!("Could not read {path:?} because of {error:?}")
            }
        }
    }
    applications
}
