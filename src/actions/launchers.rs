use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::prelude::OsStrExt;
use std::process::Command;
use std::ptr;
use std::thread::sleep;
use std::time::{Duration, UNIX_EPOCH};
use std::fs;
use winapi::shared::windef::HWND__;
use winapi::um::winuser::FindWindowW;
use regex::Regex;
use ini::Ini;
use directories::BaseDirs;
use execute::Execute;
use crate::structs;

pub fn launch_game(cfg: &structs::SeederConfig, game_id: &str, role: &str, old_game_id: &str) {
    if cfg.use_ea_desktop {
        return launch_game_ea_desktop(cfg, game_id, role, old_game_id);
    }
    println!("Launching game after EA Desktop startup...");
    launch_game_origin(cfg, game_id, role)
}

pub fn launch_game_ea_desktop(cfg: &structs::SeederConfig, game_id: &str, role: &str, old_game_id: &str) {
    // it needs to restart launcher
    if game_id != old_game_id {
        stop_ea_desktop();
        edit_ea_desktop(format!(
            "-webMode MP -Origin_NoAppFocus --activate-webhelper -requestState State_ClaimReservation -gameId {} -gameMode MP -role {} -asSpectator {}",
            game_id,
            role,
            &(role == "spectator").to_string()[..],
        ).into());
        start_ea_desktop();
    }

    let mut command = Command::new("cmd");
    command.args(&["/C", "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Battlefield 1\\Battlefield 1.lnk"]);
    match command.execute() {
        Ok(_) => println!("game launched"),
        Err(e) => println!("failed to launch game: {}", e),
    }
}

pub fn launch_game_origin(cfg: &structs::SeederConfig, game_id: &str, role: &str) {
    let mut command = Command::new(cfg.game_location.clone());
    if cfg.usable_client {
        command.args([
            "-webMode",
            "MP",
            "-Origin_NoAppFocus",
            "--activate-webhelper",
            "-requestState",
            "State_ClaimReservation",
            "-gameId",
            game_id,
            "-gameMode",
            "MP",
            "-role",
            role,
            "-asSpectator",
            &(role == "spectator").to_string()[..],
        ]);
    } else {
        command.args([
            "-Window.Fullscreen",
            "false",
            "-RenderDevice.MinDriverRequired",
            "false",
            "-Core.HardwareGpuBias",
            "-1",
            "-Core.HardwareCpuBias",
            "-1",
            "-Core.HardwareProfile",
            "Hardware_Low",
            "-RenderDevice.CreateMinimalWindow",
            "true",
            "-RenderDevice.NullDriverEnable",
            "true",
            "-Texture.LoadingEnabled",
            "false",
            "-Texture.RenderTexturesEnabled",
            "false",
            "-Client.TerrainEnabled",
            "false",
            "-Decal.SystemEnable",
            "false",
            "-webMode",
            "MP",
            "-Origin_NoAppFocus",
            "--activate-webhelper",
            "-requestState",
            "State_ClaimReservation",
            "-gameId",
            game_id,
            "-gameMode",
            "MP",
            "-role",
            role,
            "-asSpectator",
            &(role == "spectator").to_string()[..],
        ]);
    }
    match command.spawn() {
        Ok(_) => println!("game launched"),
        Err(e) => println!("failed to launch game: {}", e),
    }
}

pub fn is_launcher_running(cfg: &structs::SeederConfig) -> structs::GameInfo {
    if cfg.use_ea_desktop {
        return is_ea_desktop_running();
    }
    is_origin_running()
}

pub fn is_origin_running() -> structs::GameInfo {
    unsafe {
        let window: Vec<u16> = OsStr::new("Origin")
            .encode_wide()
            .chain(once(0))
            .collect();
        let window_handle = FindWindowW(std::ptr::null_mut(), window.as_ptr());
        let no_game: *mut HWND__ = ptr::null_mut();
        structs::GameInfo {
            is_running: window_handle != no_game,
            game_process: window_handle,
        }
    }
}

pub fn is_ea_desktop_running() -> structs::GameInfo {
    unsafe {
        let window: Vec<u16> = OsStr::new("EA")
            .encode_wide()
            .chain(once(0))
            .collect();
        let window_handle = FindWindowW(std::ptr::null_mut(), window.as_ptr());
        let no_game: *mut HWND__ = ptr::null_mut();
        structs::GameInfo {
            is_running: window_handle != no_game,
            game_process: window_handle,
        }
    }
}

pub fn restart_launcher(cfg: &structs::SeederConfig) {
    if cfg.use_ea_desktop {
        return restart_ea_desktop();
    }
    restart_origin()
}

pub fn restart_ea_desktop() {
    println!("Restarting EA Desktop");
    stop_ea_desktop();
    start_ea_desktop();
}

pub fn stop_ea_desktop() {
    let ea_desktop_process = winproc::Process::from_name("EADesktop.exe");
    match ea_desktop_process {
        Ok(mut process) => match process.terminate(1) {
            Ok(_) => {
                println!("Closed EA Desktop");
                sleep(Duration::from_secs(10));
            }
            Err(e) => println!("failed to close EA Desktop (likely permissions): {}", e),
        },
        Err(_) => {
            println!("EA Desktop not found!");
        }
    }
}

pub fn start_ea_desktop() {
    let mut command = Command::new("cmd");
    command.args(&["/C", "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\EA\\EA.lnk"]);
    match command.execute() {
        Ok(_) => {
            println!("EA Desktop launched");
            sleep(Duration::from_secs(40));
        },
        Err(e) => println!("Failed to launch EA Desktop: {}", e),
    }
}

pub fn restart_origin() {
    println!("Restarting Origin");
    let origin_process = winproc::Process::from_name("Origin.exe");
    let mut command = Command::new("C:\\Program Files (x86)\\Origin\\Origin.exe");
    match origin_process {
        Ok(mut process) => match process.terminate(1) {
            Ok(_) => {
                println!("Closed Origin");
                sleep(Duration::from_secs(10));
            }
            Err(e) => println!("failed to close origin (likely permissions): {}", e)
        },
        Err(_) => {
            println!("origin not found!");
        }
    }
    match command.spawn()
    {
        Ok(_) => {
            println!("origin launched");
            sleep(Duration::from_secs(20));
        },
        Err(e) => println!("failed to launch origin: {}", e),
    }
}

pub fn edit_ea_desktop(launch_settings: String) {
    println!("Changing EA Desktop config...");
    let base_dirs = match BaseDirs::new() {
        Some(base) => base,
        None => return println!("Generic base dir gather failure, are you not on Windows?"),
    };
    let appdata_local = match base_dirs.data_local_dir().to_str() {
        Some(appdata) => appdata,
        None => return println!("AppData dir not found, are you not on Windows?"),
    };
    let paths = match fs::read_dir(appdata_local.to_owned() + "\\Electronic Arts\\EA Desktop") {
        Ok(paths) => paths,
        Err(_) => return println!("EA Desktop folder not found in AppData!"),
    };

    let mut newest_file = structs::EaDesktopNewestFile { time: 0, location: "".into(), file_name: "".into() };
    let re = match Regex::new(r"^user_.*.ini$") {
        Ok(re) => re,
        Err(_) => return println!("Invalid REGEX for gathering EA desktop"),
    };
    for path_result in paths {

        
        if let Ok(path) = path_result {
            // check filename errors
            match path.file_name().to_str() {
                Some(name) => {

                    // get modified time in secs
                    match path.metadata() {
                        Ok(e) => match e.modified() {
                            Ok(e) => match e.duration_since(UNIX_EPOCH) {
                                Ok(e) => {
                                    let timestamp = e.as_secs();

                                    // check if newer and use only .ini files
                                    if re.is_match(name) && timestamp > newest_file.time {

                                        // set to newest if true
                                        match path.path().to_str() {
                                            Some(location) => {
                                                newest_file = structs::EaDesktopNewestFile {
                                                    file_name: name.to_owned(),
                                                    time: timestamp.to_owned(),
                                                    location: location.to_owned(),
                                                }
                                            },
                                            None => continue,
                                        };
                                    }
                                }
                                Err(_) => continue,
                            },
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    };
                },
                None => continue,
            };

        }
    }

    if newest_file.file_name != "" {
        println!("Using EA Desktop config file: {}", newest_file.file_name);
    } else {
        return println!("Failed to find config file for ea launcher, please login first!");
    }

    let mut new_conf = Ini::new();
    let old_conf = match Ini::load_from_file(newest_file.location.clone()) {
        Ok(conf) => conf,
        Err(e) => return println!("Failed to load file: {}", e),
    };
    let old_section = match old_conf.section(None::<String>) {
        Some(section) => section,
        None => return println!("Empty EA Desktop config file!"),
    };
    new_conf.with_section(None::<String>).set("user.gamecommandline.origin.ofr.50.0002683", "");
    
    // copy old config
    for (key, value) in old_section.iter() {
        match new_conf.section_mut(None::<String>) {
            Some(conf) => {
                if key == "user.gamecommandline.origin.ofr.50.0000557" {
                    // add launch params
                    conf.insert(key, launch_settings.clone());
                } else {
                    conf.insert(key, value)
                }
            },
            None => println!("Failed to copy {:?}:{:?}", key, value),
        };
    }
    match new_conf.section_mut(None::<String>) {
        Some(conf) => {
            if !conf.contains_key("user.gamecommandline.origin.ofr.50.0000557") {
                println!("Game not found in config, please launch the game once first.")
            }
        },
        None => {},
    };

    match new_conf.write_to_file(newest_file.location) {
        Ok(_) => {},
        Err(e) => println!("Failed to save new EA Desktop config: {}", e),
    };
}
