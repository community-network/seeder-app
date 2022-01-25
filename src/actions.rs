use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::prelude::OsStrExt;
use std::process::Command;
use std::ptr;
use std::thread::sleep;
use std::time::Duration;
use winapi::shared::windef::{HWND__, LPRECT, RECT};
use winapi::um::winuser::{
    FindWindowW, GetDesktopWindow, GetWindowRect, SetForegroundWindow, ShowWindow,
};

use crate::chars::{char_to_dxcodes, DXCode};
use crate::{send_keys, structs};

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

pub fn anti_afk() {
    let game_info = is_running();
        if game_info.is_running {
            unsafe {
                SetForegroundWindow(game_info.game_process);
                ShowWindow(game_info.game_process, 9);
                sleep(Duration::from_millis(1808));
                send_keys::key_enter(0x12, 200);
                sleep(Duration::from_millis(100));
                ShowWindow(game_info.game_process, 6);
            }
        }
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

pub fn ping_backend(cfg: &structs::SeederConfig, game_info: &structs::GameInfo) {
    match ureq::post("https://manager-api.gametools.network/api/seederinfo").send_json(
        ureq::json!({
            "groupid": cfg.group_id,
            "isrunning": game_info.is_running,
            "hostname": cfg.hostname
        }),
    ) {
        Ok(_) => {}
        Err(_) => println!("Couln't send update of client to backend"),
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

pub fn quit_game() {
    println!("Quitting old session..");
    let game_process = winproc::Process::from_name("bf1.exe");
    match game_process {
        Ok(mut process) => match process.terminate(1) {
            Ok(_) => println!("closed the game"),
            Err(e) => {
                println!("failed to close game (likely permissions): {}", e);
            }
        },
        Err(_) => {
            println!("no game process found!");
        }
    }
}

pub fn restart_origin() {
    println!("Restarting Origin");
    let game_process = winproc::Process::from_name("Origin.exe");
    let mut command = Command::new("C:\\Program Files (x86)\\Origin\\Origin.exe");
    match game_process {
        Ok(mut process) => match process.terminate(1) {
            Ok(_) => println!("Closed Origin"),
            Err(e) => {
                println!("failed to close origin (likely permissions): {}", e);
            }
        },
        Err(_) => {
            println!("origin not found!");
        }
    }
    match command.spawn()
    {
        Ok(_) => println!("origin launched"),
        Err(e) => println!("failed to launch origin: {}", e),
    }
}

pub fn launch_game(cfg: &structs::SeederConfig, game_id: &str, role: &str) {
    println!("joining id: {}", game_id);
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
            "-DebrisSystem.Enable",
            "false",
            "-Render.DebugRendererEnable",
            "false",
            "-Core.HardwareGpuBias",
            "-1",
            "-Core.HardwareCpuBias",
            "-1",
            "-Texture.LoadingEnabled",
            "false",
            "-Client.TerrainEnabled",
            "false",
            "-Mesh.LoadingEnabled",
            "false",
            "-Core.HardwareProfile",
            "Hardware_Low",
            "-RenderDevice.CreateMinimalWindow",
            "true",
            "-RenderDevice.NullDriverEnable",
            "true",
            "-GameTime.ForceUseSleepTimer",
            "true",
            "-Mesh.LoadingEnabled",
            "false",
            "-ShaderSystem.DatabaseLoadingEnable",
            "false",
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
