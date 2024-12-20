use crate::actions::launchers;
use crate::input::{
    chars::{char_to_dxcodes, DXCode},
    send_keys,
};
use crate::structs;
use crate::structs::GameInfo;
use registry::{Hive, Security};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::prelude::OsStrExt;
use std::ptr;
use std::sync::atomic::AtomicU32;
use std::sync::{atomic, Arc};
use std::thread::sleep;
use std::time::Duration;
use winapi::shared::windef::{HWND__, LPRECT, RECT};
use winapi::um::winuser::IsIconic;
use winapi::um::winuser::{
    FindWindowW, GetDesktopWindow, GetForegroundWindow, GetWindowRect, SendMessageW,
    SetForegroundWindow, ShowWindow,
};

pub fn is_fullscreen(cfg: &structs::SeederConfig) -> bool {
    let game_info = is_running(cfg);
    if game_info.is_running {
        let mut rect = RECT {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        };
        let game_size = LPRECT::from(&mut rect.clone());
        let screen_size = LPRECT::from(&mut rect);
        unsafe {
            GetWindowRect(game_info.game_process, game_size);
            GetWindowRect(GetDesktopWindow(), screen_size);
            ((*game_size).left == (*screen_size).left)
                && ((*game_size).right == (*screen_size).right)
                && ((*game_size).top == (*screen_size).top)
                && ((*game_size).bottom == (*screen_size).bottom)
        }
    } else {
        false
    }
}

pub fn find_game(cfg: &structs::SeederConfig) -> String {
    match Hive::LocalMachine.open(
        format!("SOFTWARE\\Wow6432Node\\EA Games\\{}", cfg.game.full_name()),
        Security::Read,
    ) {
        Ok(regkey) => match regkey.value("Install Dir") {
            Ok(result) => format!("{}\\{}", result, cfg.game.process_start()),
            Err(_) => {
                log::warn!(
                    "{} not found in ea desktop's registry, using default origin location.",
                    cfg.game.full_name()
                );
                format!(
                    "C:\\Program Files (x86)\\Origin Games\\{}\\{}",
                    cfg.game.full_name(),
                    cfg.game.process_start()
                )
            }
        },
        Err(_) => {
            log::warn!(
                "{} not found in ea desktop's registry, using default origin location.",
                cfg.game.full_name()
            );
            format!(
                "C:\\Program Files (x86)\\Origin Games\\{}\\{}",
                cfg.game.full_name(),
                cfg.game.process_start()
            )
        }
    }
}

pub fn minimize(game_info: &GameInfo) {
    unsafe {
        // check minimized or minimize
        if IsIconic(game_info.game_process) == 0 {
            ShowWindow(game_info.game_process, 6);
        }
    }
}

pub fn anti_afk(cfg: &structs::SeederConfig) {
    let game_info = is_running(cfg);
    if game_info.is_running {
        minimize(&game_info);
        log::debug!("Running anti-afk script...");
        unsafe {
            let current_forground_window = GetForegroundWindow();
            let l_param = make_l_param(20, 20);
            SendMessageW(game_info.game_process, 0x201, 0, l_param as isize);
            SendMessageW(game_info.game_process, 0x202, 0, l_param as isize);
            SetForegroundWindow(current_forground_window);
        }
    }
}

fn make_l_param(lo_word: i32, hi_word: i32) -> i32 {
    (hi_word << 16) | (lo_word & 0xffff)
}

pub fn send_message(to_send: String, cfg: &structs::SeederConfig) {
    let game_info = is_running(cfg);
    log::debug!("Sending message: {}", to_send);
    if game_info.is_running {
        unsafe {
            SetForegroundWindow(game_info.game_process);
            ShowWindow(game_info.game_process, 9);
            sleep(Duration::from_millis(5000));
            send_keys::key_enter(0x24, 80);
            sleep(Duration::from_millis(2000));
            let mut message: Vec<DXCode> = Vec::new();
            for char in to_send.chars() {
                if let Some(dx) = char_to_dxcodes(char) {
                    message.push(dx)
                }
            }
            send_keys::send_string(message);
            sleep(Duration::from_millis(100));
            send_keys::key_enter(0x1C, 80);
            sleep(Duration::from_millis(2500));
            ShowWindow(game_info.game_process, 6);
        }
    }
}

pub fn is_running(cfg: &structs::SeederConfig) -> structs::GameInfo {
    unsafe {
        let game_window: Vec<u16> = OsStr::new(cfg.game.window_name())
            .encode_wide()
            .chain(once(0))
            .collect();
        let game_window_handle = FindWindowW(std::ptr::null_mut(), game_window.as_ptr());
        let anticheat_process = winproc::Process::from_name("EAAntiCheat.GameServiceLauncher.exe");

        let no_game: *mut HWND__ = ptr::null_mut();
        structs::GameInfo {
            is_running: game_window_handle != no_game,
            game_process: game_window_handle,
            anticheat_launcher_running: anticheat_process.is_ok(),
        }
    }
}

pub fn quit(
    cfg: &structs::SeederConfig,
    game_running: &Arc<AtomicU32>,
    retry_launch: &Arc<AtomicU32>,
) {
    log::info!("Quitting old session..");
    let game_process = winproc::Process::from_name(cfg.game.process_name());
    match game_process {
        Ok(mut process) => match process.terminate(1) {
            Ok(_) => {
                log::info!("closed the game");

                if cfg.launcher == structs::Launchers::EADesktop {
                    log::info!("waiting 5 seconds for game to close...");
                    sleep(Duration::from_secs(5));
                    log::info!("ready!");
                }

                game_running.store(0, atomic::Ordering::Relaxed);
                retry_launch.store(0, atomic::Ordering::Relaxed);
            }
            Err(e) => {
                log::error!("failed to close game (likely permissions): {}", e);
            }
        },
        Err(_) => {
            log::info!("no game process found!");
        }
    }

    // quit ea desktop to reset config
    if cfg.launcher == structs::Launchers::EADesktop {
        launchers::stop_ea_desktop();
    }
}

pub fn launch(
    cfg: &structs::SeederConfig,
    game_id: &str,
    role: &str,
    game_running: &Arc<AtomicU32>,
    retry_launch: &Arc<AtomicU32>,
) {
    if game_running.load(atomic::Ordering::Relaxed) == 1 {
        // if it tried to launch but failed twice
        if retry_launch.load(atomic::Ordering::Relaxed) == 10 {
            launchers::restart_launcher(cfg);
            // make retries 0
            retry_launch.store(0, atomic::Ordering::Relaxed);
        } else {
            // if failed once
            retry_launch.fetch_add(1, atomic::Ordering::Relaxed);
        }
    }
    log::info!("joining id: {}", game_id);

    launchers::launch_game(cfg, game_id, role)
}
