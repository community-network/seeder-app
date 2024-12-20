use crate::structs::{self, Games};
use directories::BaseDirs;
use ini::Ini;
use regex::Regex;
use registry::{Hive, Security};
use std::ffi::OsStr;
use std::fs;
use std::iter::once;
use std::os::windows::prelude::OsStrExt;
use std::process::Command;
use std::ptr;
use std::thread::sleep;
use std::time::{Duration, UNIX_EPOCH};
use winapi::shared::windef::HWND__;
use winapi::um::winuser::FindWindowW;

pub fn launch_game(cfg: &structs::SeederConfig, game_id: &str, role: &str) {
    match cfg.launcher {
        structs::Launchers::EADesktop => {
            log::info!("Launching game after EA Desktop startup...");
            launch_game_ea_desktop(cfg, game_id, role)
        }
        structs::Launchers::Origin => launch_game_origin(cfg, game_id, role),
        structs::Launchers::Steam => launch_game_steam(cfg, game_id, role),
    }
}

pub fn find_link2ea() -> String {
    match Hive::LocalMachine.open(
        "SOFTWARE\\Wow6432Node\\Electronic Arts\\EA Desktop",
        Security::Read,
    ) {
        Ok(regkey) => match regkey.value("ClientPath") {
            Ok(result) => result.to_string().replace("EADesktop.exe", "Link2EA.exe"),
            Err(_) => {
                log::warn!("Link2EA.exe not found in registry, using default Link2EA location.");
                "C:\\Program Files\\Electronic Arts\\EA Desktop\\EA Desktop\\Link2EA.exe".to_owned()
            }
        },
        Err(_) => {
            log::warn!("Link2EA.exe not found in registry, using default Link2EA location.");
            "C:\\Program Files\\Electronic Arts\\EA Desktop\\EA Desktop\\Link2EA.exe".to_owned()
        }
    }
}

pub fn start_wait_timer(cfg: &structs::SeederConfig) {
    let mut timeout = 0;
    let mut anticheat_running = false;
    let mut running = false;
    while !running {
        if timeout > 10 {
            // give up on to many tries waiting and continue anyway
            log::warn!("waiting to long, continueing..");
            break;
        }

        log::debug!("Testing if the game started");

        let start_info = super::game::is_running(cfg);
        running = start_info.is_running;
        if cfg.game == Games::Bf1 && start_info.anticheat_launcher_running && !anticheat_running {
            log::info!("Anticheat is running");
            timeout = 0;
            anticheat_running = true;
        }
        sleep(Duration::from_secs(5));
        timeout += 1;
    }
    log::info!("Game started");
}

pub fn launch_game_ea_desktop(cfg: &structs::SeederConfig, game_id: &str, role: &str) {
    // it needs to restart launcher
    stop_ea_desktop();
    sleep(Duration::from_secs(5));
    let join_config = match cfg.game {
        structs::Games::Bf4 => format!(
            "-gameId {} -gameMode MP -role {} -asSpectator {} -joinWithParty false",
            game_id,
            role,
            &(role == "spectator").to_string()[..],
        ),
        structs::Games::Bf1 => match cfg.usable_client {
            true => format!(
                "-webMode MP -Origin_NoAppFocus --activate-webhelper -requestState State_ClaimReservation -gameId {} -gameMode MP -role {} -asSpectator {}",
                game_id,
                role,
                &(role == "spectator").to_string()[..],
            ),
            false => format!(
                "-Window.Fullscreen false -RenderDevice.MinDriverRequired false -Core.HardwareGpuBias -1 -Core.HardwareCpuBias -1 -Core.HardwareProfile Hardware_Low -RenderDevice.CreateMinimalWindow true -RenderDevice.NullDriverEnable true -Texture.LoadingEnabled false -Texture.RenderTexturesEnabled false -Client.TerrainEnabled false -Decal.SystemEnable false -webMode MP -Origin_NoAppFocus --activate-webhelper -requestState State_ClaimReservation -gameId {} -gameMode MP -role {} -asSpectator {}",
                game_id,
                role,
                &(role == "spectator").to_string()[..],
            ),
        },
    };
    edit_ea_desktop(cfg, join_config);

    let mut command = Command::new(cfg.game_location.clone());
    match command.spawn() {
        Ok(_) => log::info!("game launched"),
        Err(e) => log::error!("failed to launch game: {}", e),
    }

    start_wait_timer(cfg);

    // reset config after gamelaunch
    edit_ea_desktop(cfg, "".to_string());

    sleep(Duration::from_secs(10));
}

pub fn launch_game_origin(cfg: &structs::SeederConfig, game_id: &str, role: &str) {
    let mut command = Command::new(cfg.game_location.clone());
    match cfg.game {
        structs::Games::Bf4 => {
            command.args([
                "-gameId",
                game_id,
                "-gameMode",
                "MP",
                "-role",
                role,
                "-asSpectator",
                &(role == "spectator").to_string()[..],
                "-joinWithParty",
                "false",
            ]);
        }
        structs::Games::Bf1 => {
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
        }
    };
    log::debug!("command to join: {:#?}", command);
    match command.spawn() {
        Ok(_) => log::info!("game launched"),
        Err(e) => log::error!("failed to launch game: {}", e),
    }
    start_wait_timer(cfg);
}

pub fn launch_game_steam(cfg: &structs::SeederConfig, game_id: &str, role: &str) {
    let mut command = Command::new(cfg.link2ea_location.clone());
    match cfg.game {
        structs::Games::Bf4 => {
            command.args([
                "link2ea://launchgame/1238860?platform=steam&theme=bf4",
                "-gameId",
                game_id,
                "-gameMode",
                "MP",
                "-role",
                role,
                "-asSpectator",
                &(role == "spectator").to_string()[..],
                "-joinWithParty",
                "false",
            ]);
        }
        structs::Games::Bf1 => {
            if cfg.usable_client {
                command.args([
                    "link2ea://launchgame/1238840?platform=steam&theme=bf1",
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
                    "link2ea://launchgame/1238840?platform=steam&theme=bf1",
                    "-webMode",
                    "MP",
                    "-Origin_NoAppFocus",
                    "--activate-webhelper",
                    "-requestState",
                    "State_ClaimReservation",
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
        }
    };
    match command.spawn() {
        Ok(_) => log::info!("game launched"),
        Err(e) => log::error!("failed to launch game: {}", e),
    }
    start_wait_timer(cfg);
}

pub fn is_launcher_running(cfg: &structs::SeederConfig) -> structs::GameInfo {
    unsafe {
        let launcher_window: Vec<u16> = OsStr::new(cfg.launcher.window_name())
            .encode_wide()
            .chain(once(0))
            .collect();
        let launcher_window_handle = FindWindowW(std::ptr::null_mut(), launcher_window.as_ptr());
        let no_game: *mut HWND__ = ptr::null_mut();
        structs::GameInfo {
            is_running: launcher_window_handle != no_game,
            game_process: launcher_window_handle,
            anticheat_launcher_running: false,
        }
    }
}

pub fn restart_launcher(cfg: &structs::SeederConfig) {
    match cfg.launcher {
        structs::Launchers::EADesktop => restart_ea_desktop(),
        structs::Launchers::Origin => restart_origin(),
        structs::Launchers::Steam => restart_steam_ea_desktop(),
    }
}

pub fn restart_steam_ea_desktop() {
    log::info!("Stopping EA Desktop");
    stop_ea_desktop();
    log::info!("Restarting Steam");
    restart_steam();
}

pub fn restart_steam() {
    let steam_process = winproc::Process::from_name("steam.exe");
    let mut command = Command::new("C:\\Program Files (x86)\\Steam\\Steam.exe");
    match steam_process {
        Ok(mut process) => match process.terminate(1) {
            Ok(_) => {
                log::info!("Closed Steam");
                sleep(Duration::from_secs(10));
            }
            Err(e) => log::error!("failed to close Steam (likely permissions): {}", e),
        },
        Err(_) => {
            log::info!("Steam not found!");
        }
    }
    match command.spawn() {
        Ok(_) => {
            log::info!("Steam launched");
            sleep(Duration::from_secs(20));
        }
        Err(e) => log::error!("failed to launch Steam: {}", e),
    }
}

pub fn restart_ea_desktop() {
    log::info!("Restarting EA Desktop");
    stop_ea_desktop();
}

pub fn stop_ea_desktop() {
    match winproc::Process::all() {
        Ok(processes) => {
            for mut process in processes {
                for item in ["EADesktop.exe", "EACefSubProcess.exe"].iter() {
                    if process.name().unwrap_or_default() == item.to_string() {
                        match process.terminate(1) {
                            Ok(_) => {
                                log::info!("Closed {}", item);
                                sleep(Duration::from_secs(10));
                            }
                            Err(e) => {
                                log::error!("failed to close {} (likely permissions): {}", item, e)
                            }
                        }
                    }
                }
            }
        }
        Err(e) => log::error!("failed to access running processes, {:#?}", e),
    }
}

pub fn restart_origin() {
    log::info!("Restarting Origin");
    let origin_process = winproc::Process::from_name("Origin.exe");
    let mut command = Command::new("C:\\Program Files (x86)\\Origin\\Origin.exe");
    match origin_process {
        Ok(mut process) => match process.terminate(1) {
            Ok(_) => {
                log::info!("Closed Origin");
                sleep(Duration::from_secs(10));
            }
            Err(e) => log::error!("failed to close origin (likely permissions): {}", e),
        },
        Err(_) => {
            log::info!("origin not found!");
        }
    }
    match command.spawn() {
        Ok(_) => {
            log::info!("origin launched");
            sleep(Duration::from_secs(20));
        }
        Err(e) => log::error!("failed to launch origin: {}", e),
    }
}

pub fn edit_ea_desktop(cfg: &structs::SeederConfig, launch_settings: String) {
    if launch_settings == *"" {
        log::info!("Cleaning up EA Desktop config...");
    } else {
        log::info!("Changing EA Desktop config...");
    }
    let base_dirs = match BaseDirs::new() {
        Some(base) => base,
        None => return log::error!("Generic base dir gather failure, are you not on Windows?"),
    };
    let appdata_local = match base_dirs.data_local_dir().to_str() {
        Some(appdata) => appdata,
        None => return log::error!("AppData dir not found, are you not on Windows?"),
    };
    let paths = match fs::read_dir(appdata_local.to_owned() + "\\Electronic Arts\\EA Desktop") {
        Ok(paths) => paths,
        Err(_) => return log::error!("EA Desktop folder not found in AppData!"),
    };

    let mut newest_file = structs::EaDesktopNewestFile {
        time: 0,
        location: "".into(),
        file_name: "".into(),
    };
    let re = match Regex::new(r"^user_.*.ini$") {
        Ok(re) => re,
        Err(_) => return log::error!("Invalid REGEX for gathering EA desktop"),
    };
    for path in paths.flatten() {
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
                                        }
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
            }
            None => continue,
        };
    }

    if !newest_file.file_name.is_empty() {
        if launch_settings != *"" {
            log::info!("Using EA Desktop config file: {}", newest_file.file_name);
        }
    } else {
        return log::error!("Failed to find config file for ea launcher, please login first!");
    }

    let mut new_conf = Ini::new();
    let old_conf = match Ini::load_from_file(newest_file.location.clone()) {
        Ok(conf) => conf,
        Err(e) => return log::error!("Failed to load file: {}", e),
    };
    let old_section = match old_conf.section(None::<String>) {
        Some(section) => section,
        None => return log::error!("Empty EA Desktop config file!"),
    };
    new_conf
        .with_section(None::<String>)
        .set("user.gamecommandline.origin.ofr.50.0002683", "");

    let game_versions = cfg.game.game_versions();
    // copy old config
    for (key, value) in old_section.iter() {
        match new_conf.section_mut(None::<String>) {
            Some(conf) => {
                if game_versions.contains(&key) {
                    // add launch params
                    conf.remove(key);
                } else {
                    conf.insert(key, value)
                }
            }
            None => log::error!("Failed to copy {:?}:{:?}", key, value),
        };
    }

    if let Some(conf) = new_conf.section_mut(None::<String>) {
        for game_version in game_versions {
            conf.insert(game_version, launch_settings.clone());
        }
    }

    match new_conf.write_to_file(newest_file.location) {
        Ok(_) => {}
        Err(e) => log::error!("Failed to save new EA Desktop config: {}", e),
    };
}
