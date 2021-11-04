use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::prelude::OsStrExt;
use std::process::Command;
use std::ptr;
use winapi::shared::windef::HWND__;
use winapi::um::winuser::FindWindowW;

use crate::structs;

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

pub fn launch_game(cfg: &structs::SeederConfig, game_id: &str, role: &str) {
    println!("joining id: {}", game_id);
    let mut command = Command::new(cfg.game_location.clone());
    if role == "spectator" {
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
            "spectator",
            "-asSpectator",
            "true"
        ]);
    } else {
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
            "soldier",
            "-asSpectator",
            "false"
        ]);
    }
    match command.spawn()
    {
        Ok(_) => println!("game launched"),
        Err(e) => println!("failed to launch game: {}", e),
    }
}