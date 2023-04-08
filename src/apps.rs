use freedesktop_entry_parser::{parse_entry, Entry};
use linicon::lookup_icon;
use linicon_theme::get_icon_theme;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
const APPLICATION_PATHS: [&str; 2] = [
    "/usr/share/applications",
    "/var/lib/flatpak/exports/share/applications",
];
const ICON_PATHS: [&str; 2] = [
    "/var/lib/flatpak/exports/share/icons/hicolor/scalable/apps/",
    "/var/lib/flatpak/exports/share/applications",
];
const ICON_EXT: [&str; 1] = ["svg"];
use std::cmp::Ordering;
#[derive(Clone, Eq, PartialEq)]
enum ApplicationType {
    DESKTOPFILE,
    BINARY,
}
#[derive(Clone, Eq, PartialEq)]
pub struct Application {
    pub name: String,
    command: String,
    pub icon_path: Option<PathBuf>,
    application_type: ApplicationType,
    // entry: Option<Entry>
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
    pub fn run(&self, quit: bool) {
        println!("Running {}", self.name);
        println!("with command {}", self.command);
        let split_command: Vec<&str> = self.command.split(" ").collect();
        let mut args: Vec<&str> = Vec::new();
        for arg in split_command[1..].into_iter() {
            if *arg != "" && !arg.starts_with("%") {
                args.push(arg.to_owned());
            }
        }
        Command::new(split_command[0]).args(args).spawn().unwrap();
        if quit {
            std::process::exit(0);
        }
    }
}
fn search_icons(name: &str) {}
pub fn collect_applications() -> Vec<Application> {
    let mut applications: Vec<Application> = Vec::new();
    for path in APPLICATION_PATHS {
        let files: Vec<Result<fs::DirEntry, std::io::Error>> =
            fs::read_dir(path).unwrap().collect();
        let path_applications: Vec<Option<Application>> = files
            .par_iter()
            .map(|file_res| {
                match file_res {
                    Ok(file) => {
                        if file.file_name().to_string_lossy().contains("desktop") {
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
                                let name: &str =
                                    entry.section("Desktop Entry").attr("Name").expect(&format!(
                                        "Incomplete Desktop file {}",
                                        file.path().to_string_lossy()
                                    ));
                                let command: &str =
                                    entry.section("Desktop Entry").attr("Exec").expect(&format!(
                                        "Incomplete Desktop file {}",
                                        file.path().to_string_lossy()
                                    ));
                                let icon_path: Option<PathBuf> =
                                    match entry.section("Desktop Entry").attr("Icon") {
                                        Some(path) => match lookup_icon(path.to_owned())
                                            .from_theme(get_icon_theme().unwrap())
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
                                Some(Application {
                                    name: name.into(),
                                    command: command.into(),
                                    icon_path: icon_path,
                                    application_type: ApplicationType::DESKTOPFILE,
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Err(error) => {
                        println!("Error encountered while reading file {:?}", error);
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
    return applications;
}
