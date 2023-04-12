use freedesktop_entry_parser::{parse_entry, Entry};
use linicon::lookup_icon;
use linicon_theme::get_icon_theme;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
/// The paths where the desktop files and binaries are located. Will be exported to a config file
/// and inserted in the defaults
const APPLICATION_PATHS: [&str; 4] = [
    "/usr/share/applications",
    "/usr/local/share/applications",
    "$HOME/.local/share/applications",
    "/var/lib/flatpak/exports/share/applications",
];

use std::cmp::Ordering;
/// The type of application. Either a binary (not yet supported) or a Desktop file
#[derive(Clone, Eq, PartialEq)]
enum ApplicationType {
    DESKTOPFILE,
    // BINARY,
}
/// A specific application found on the system
#[derive(Clone, Eq, PartialEq)]
pub struct Application {
    /// Name of the application as stated in the desktop file or the name of the executable if
    /// Application Type is binary
    pub name: String,
    /// The command to execute. Either the entry 'Exec' in the Desktop file or path to executable
    command: String,
    /// Optional icon path, if defined in the desktop file and found in the system
    pub icon_path: Option<PathBuf>,
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
        // println!("command {:?}", self.command);
        let split_command: Vec<&str> = self.command.split(" ").collect();
        let mut args: Vec<&str> = Vec::new();
        for arg in split_command[1..].into_iter() {
            if *arg != "" && !arg.starts_with("%") {
                args.push(arg.to_owned());
            }
        }
        Command::new(split_command[0].trim_matches('\"'))
            .args(args)
            .spawn()
            .unwrap();
        if quit {
            std::process::exit(0);
        }
    }
}
// fn search_icons(name: &str) {}
/// Find applications in the APPLICATION_PATHS and return them as a `Vec<Application>`
pub fn collect_applications(get_icons: bool) -> Vec<Application> {
    let mut applications: Vec<Application> = Vec::new();

    let icon_theme: String = get_icon_theme().unwrap();
    for path in APPLICATION_PATHS {
        println!("{path:?}");

        match fs::read_dir(path) {
            Ok(files) => {
                let path_applications: Vec<Option<Application>> = files
                    .collect::<Vec<Result<fs::DirEntry, std::io::Error>>>()
                    .par_iter()
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
                                        let icon_path: Option<PathBuf> = if get_icons {
                                            match entry.section("Desktop Entry").attr("Icon") {
                                                Some(path) => match lookup_icon(path.to_owned())
                                                    .from_theme(icon_theme.clone())
                                                    .with_size(8)
                                                    .next()
                                                {
                                                    Some(icon_path) => match icon_path {
                                                        Ok(linicon_path) => {
                                                            Some(linicon_path.path.clone())
                                                        }
                                                        Err(_) => None,
                                                    },
                                                    None => None,
                                                },

                                                None => None,
                                            }
                                        } else {
                                            None
                                        };
                                        Some(Application {
                                            name: name.into(),
                                            command: command.into(),
                                            icon_path,
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
            Err(error) => {
                println!("Could not read {path:?} because of {error:?}")
            }
        }
    }
    return applications;
}
/// Clear the Matches and then from the vector of applications fuzzy find the search_str and  append to the matches
pub fn find_application(
    search_str: &str,
    applications: &Vec<Application>,
    matches: &mut Vec<(Application, i64)>,
) {
    let matcher = SkimMatcherV2::default();
    matches.clear();
    for application in applications {
        let search_match: Option<i64> = matcher.fuzzy_match(&application.name, search_str);
        println!(
            "{} = {} : {:?}",
            search_str, &application.name, search_match
        );
        match search_match {
            Some(score) => matches.push((application.clone(), score)),
            None => {}
        }
    }
    matches.sort_by(|a, b| b.1.cmp(&a.1));
}
