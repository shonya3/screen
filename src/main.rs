use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::Result;

use chrono::Local;
use screenshots::Screen;

fn main() {
    // Prepare paths
    let current_dir = env::current_dir().unwrap();
    let screen_dir = current_dir.join("data").join("queue");

    if !screen_dir.exists() {
        fs::create_dir_all(&screen_dir).unwrap();
    }

    // Prepare image path
    let local_time = Local::now().format("%Y-%m-%d-%H-%M-%S");
    let filename = format!("{local_time}.png");
    let image_path = screen_dir.join(&filename);

    // Choosing the preferred screen
    let poe_config_path = poe_config_path().unwrap();
    let monitor_number_from_poe_config =
        active_monitor_number_from_poe_config(poe_config_path).unwrap();
    let screen = choose_actual_monitor(monitor_number_from_poe_config).unwrap();

    // Make screenshot
    let image = screen.capture().unwrap();
    image.save(&image_path).unwrap();
}

fn choose_actual_monitor(monitor_number_from_poe_config: Option<usize>) -> Result<Screen> {
    // default index
    let mut index = 0;

    // if some number in poe config, set this to index
    if let Some(number_from_poe_config) = monitor_number_from_poe_config {
        index = number_from_poe_config;
    }

    let screens = Screen::all()?;
    let number_of_screens = screens.len();

    // If u set the second monitor as active and then turned it off,
    // your poe config file wil still point to this second monitor.
    // If there are not enough screens, fallback to 0-index.
    if number_of_screens <= index {
        index = 0;
    }

    Ok(screens[index])
}

/// cross-platform C:\Users\username\Documents\My Games\Path of Exile\production_Config.ini
fn poe_config_path() -> Option<PathBuf> {
    dirs_next::document_dir().map(|docs_dir| {
        docs_dir
            .join("My Games")
            .join("Path of Exile")
            .join("production_Config.ini")
    })
}

/// Read poe production_Config.ini, try to find the index of the preferred minitor.
/// Something like this:
///   
///  adapter_name=AMD Radeon RX 5700 XT(#0)
///
fn active_monitor_number_from_poe_config<P>(poe_config_path: P) -> Result<Option<usize>>
where
    P: AsRef<Path>,
{
    let file = File::open(poe_config_path)?;
    for line in BufReader::new(file).lines() {
        if let Ok(line) = line {
            if line.contains("adapter_name") {
                let Some(start) = line.rfind("(#") else {
                    return Ok(None);
                };
                let Some(end) = line.rfind(")") else {
                    return Ok(None);
                };

                let Some(substr) = line.get(start + 2..end) else {
                    return Ok(None);
                };

                let monitor_index = substr.parse::<usize>()?;

                return Ok(Some(monitor_index));
            }
        }
    }

    Ok(None)
}
