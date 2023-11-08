#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![deny(clippy::print_stdout)] // #![feature(map_try_insert)]
mod apps;
mod config;
mod egui_ui;
mod iced_ui;
use crate::apps::ApplicationManager;
use crate::config::{load_config, Config};
#[cfg(feature = "egui-ui")]
use crate::egui_ui::ui::launch_egui_ui;
#[cfg(feature = "iced-ui")]
use crate::iced_ui::iced_ui::launch_iced_ui;
use clap::Parser;
use config::UIFramework;
use log::{debug, error};
use single_instance::SingleInstance;
use std::error::Error;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use std::{io, io::prelude::*, sync::mpsc, sync::mpsc::Receiver, thread};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(long)]
    select_from_stdin: bool,
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();
    let mut custom_inputs: Vec<String> = Vec::new();
    if args.select_from_stdin {
        fetch_custom_commands(&mut custom_inputs)?;
    }

    let instance = SingleInstance::new("Aphorme").unwrap();
    let _ = env_logger::builder()
        .target(env_logger::Target::Stderr)
        .try_init();
    if instance.is_single() {
        let cfg: Config = load_config(None);
        let application_manager: ApplicationManager = ApplicationManager::new(
            cfg.app_cfg.unwrap_or_default(),
            cfg.gui_cfg.icon,
            instance,
            custom_inputs,
        );
        let gui_framework: UIFramework = cfg.gui_cfg.ui_framework.unwrap_or_default();
        // let gui_framework: GuiFramework = GuiFramework::EGUI; //cfg.ui_framework.unwrap_or_default();
        match gui_framework {
            UIFramework::Egui => {
                #[cfg(feature = "egui-ui")]
                match launch_egui_ui(cfg.gui_cfg, application_manager) {
                    Ok(()) => {}
                    Err(error) => error!("{error:?}"),
                };
                #[cfg(not(feature = "egui-ui"))]
                panic!("Trying to use egui without \"ui-egui\"-feature activated");
            }

            UIFramework::Iced => {
                #[cfg(feature = "iced-ui")]
                launch_iced_ui(cfg.gui_cfg, application_manager);
            }
        }
    } else {
        error!("another instance is already running");
    }
    Ok(())
}
/// Gets custom inputs piped into the program
/// If present this will replace the default applications and output the selection to stdout
fn fetch_custom_commands(custom_inputs: &mut Vec<String>) -> Result<(), RecvTimeoutError> {
    let stdin_channel = spawn_stdin_channel();

    let key = stdin_channel.recv_timeout(Duration::from_secs(1))?;
    for line in key.split('\n') {
        let command: String = line.to_string().replace('\n', "");
        if !command.is_empty() {
            custom_inputs.push(command);
        }
    }
    Ok(())
}
/// Thread which tries to read from the stin
fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().lock().read_to_end(&mut buffer).unwrap();
        if let Err(error) = tx.send(String::from_utf8_lossy(buffer.as_slice()).to_string()) {
            debug!("Unable to send {:#?}, due to {:?}", buffer, error);
            break;
        }
    });
    rx
}
