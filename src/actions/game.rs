use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::prelude::OsStrExt;
use std::ptr;
use std::thread::sleep;
use std::time::Duration;
use std::sync::{atomic, Arc};
use std::sync::atomic::AtomicU32;
use winapi::shared::windef::{HWND__, LPRECT, RECT};
use winapi::um::winuser::{
    FindWindowW, GetDesktopWindow, GetWindowRect, SetForegroundWindow, ShowWindow, GetForegroundWindow, SendMessageW
};
use crate::actions::launchers;
use crate::input::{chars::{char_to_dxcodes, DXCode}, send_keys};
use crate::structs;
use crate::structs::GameInfo;
use winapi::um::winuser::IsIconic;

pub fn is_fullscreen() -> bool {
    let game_info = is_running();
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
            return ((*game_size).left == (*screen_size).left)
                && ((*game_size).right == (*screen_size).right)
                && ((*game_size).top == (*screen_size).top)
                && ((*game_size).bottom == (*screen_size).bottom);
        }
    } else {
        false
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

pub fn anti_afk() {
    let game_info = is_running();
    if game_info.is_running {
        minimize(&game_info);
        // check minimized here??
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
    return (hi_word << 16) | (lo_word & 0xffff);
}

pub fn send_message(to_send: &String) {
    let game_info = is_running();
    if game_info.is_running {
        unsafe {
            // println!("open");
            SetForegroundWindow(game_info.game_process);
            ShowWindow(game_info.game_process, 9);
            // println!("wait");
            sleep(Duration::from_millis(5000));
            // println!("open menu");
            send_keys::key_enter(0x24, 80);
            // println!("wait");
            sleep(Duration::from_millis(2000));
            // println!("type message");
            let mut message: Vec<DXCode> = Vec::new();
            for char in to_send.chars() {
                match char_to_dxcodes(char) {
                    Some(dx) => message.push(dx),
                    None => {}
                }
            }
            send_keys::send_string(message);
            sleep(Duration::from_millis(100));
            // println!("send enter");
            send_keys::key_enter(0x1C, 80);
            sleep(Duration::from_millis(2500));
            // println!("minimize");
            ShowWindow(game_info.game_process, 6);
        }
    }
}

pub fn is_running() -> structs::GameInfo {
    unsafe {
        let window: Vec<u16> = OsStr::new("Battlefield™ 1")
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

pub fn quit(cfg: &structs::SeederConfig, game_running: &Arc<AtomicU32>, retry_launch: &Arc<AtomicU32>) {
    println!("Quitting old session..");
    let game_process = winproc::Process::from_name("bf1.exe");
    match game_process {
        Ok(mut process) => match process.terminate(1) {
            Ok(_) => {
                println!("closed the game");

                if cfg.use_ea_desktop {
                    println!("waiting 5 seconds for game to close...");
                    sleep(Duration::from_secs(5));
                    println!("ready!");
                }

                game_running.store(0, atomic::Ordering::Relaxed);
                retry_launch.store(0, atomic::Ordering::Relaxed);
            }
            Err(e) => {
                println!("failed to close game (likely permissions): {}", e);
            }
        },
        Err(_) => {
            println!("no game process found!");
        }
    }
}

pub fn launch(cfg: &structs::SeederConfig, game_id: &str, role: &str, 
    game_running: &Arc<AtomicU32>, retry_launch: &Arc<AtomicU32>, old_game_id: &str) {
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
    println!("joining id: {}", game_id);

    launchers::launch_game(cfg, game_id, role, old_game_id)
}
