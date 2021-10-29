use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::prelude::OsStrExt;
use std::process::Command;
use std::ptr;
use std::{
    mem,
    sync::{atomic, Arc},
    thread::{self, sleep},
    time::Duration,
};
use system_shutdown::shutdown;
use winapi::shared::windef::HWND__;
use winapi::um::winuser::{FindWindowW, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, SendInput, SetForegroundWindow, ShowWindow};
#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize, Debug)]
struct BroadcastMessage {
    action: String,
    gameid: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct SeederConfig {
    group_id: String,
    game_location: String,
    hostname: String,
    allow_shutdown: bool
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
struct CurrentServer {
    #[serde(rename = "gameId")]
    game_id: String,
    #[serde(rename = "groupId")]
    group_id: String,
    #[serde(rename = "timeStamp")]
    timestamp: i64,
    action: String,
    rejoin: bool,
}

struct GameInfo {
    is_running: bool,
    game_process: *mut HWND__
}

/// `SeederConfig` implements `Default`
impl ::std::default::Default for SeederConfig {
    fn default() -> Self {
        Self {
            hostname: hostname::get().unwrap().into_string().unwrap(),
            group_id: "0fda8e4c-5be3-11eb-b1da-cd4ff7dab605".into(),
            game_location: "C:\\Program Files (x86)\\Origin Games\\Battlefield 1\\bf1.exe".into(),
            allow_shutdown: false
        }
    }
}

fn main() {
    let game_running = Arc::new(atomic::AtomicU32::new(0));
    let game_running_clone = Arc::clone(&game_running);
    // anti afk thread, runs when game is in "joined" state
    thread::spawn(move || loop {
        if game_running_clone.load(atomic::Ordering::Relaxed) == 1 {
            let game_info = is_running();
            if game_info.is_running {
                unsafe {
                    // if game is not running
                    SetForegroundWindow(game_info.game_process);
                    ShowWindow(game_info.game_process, 9);
                    sleep(Duration::from_millis(1808));
                    key_enter(0x45);
                    sleep(Duration::from_millis(100));
                    ShowWindow(game_info.game_process, 6);
                }
            }
        }
        sleep(Duration::from_secs(120));
    });
    let cfg: SeederConfig = match confy::load_path("config.txt") {
        Ok(config) => config,
        Err(e) => {
            println!("error in config.txt: {}", e);
            println!("changing back to default..");
            SeederConfig {
                hostname: hostname::get().unwrap().into_string().unwrap(),
                group_id: "0fda8e4c-5be3-11eb-b1da-cd4ff7dab605".into(),
                game_location: "C:\\Program Files (x86)\\Origin Games\\Battlefield 1\\bf1.exe".into(),
                allow_shutdown: false
            }
        }
    };
    
    let mut old_seeder_info = CurrentServer{game_id: "".into(), action: "leaveServer".into(), group_id: cfg.group_id.clone(), timestamp: chrono::Utc::now().timestamp(), rejoin: true};
    confy::store_path("config.txt", cfg.clone()).unwrap();
    let connect_addr = format!(
        "https://manager-api.gametools.network/api/getseeder?groupid={}",
        cfg.group_id
    );
    println!("firing of latest request found (default on startup script)");
    loop {
        match ureq::get(&connect_addr[..]).call() {
            Ok(response) => {
                match response.into_json::<CurrentServer>() {
                    Ok(seeder_info) => {
                        let game_info = is_running();
                        let a_hour = seeder_info.timestamp < chrono::Utc::now().timestamp()-3600; // if it is older than 1 hour, dont try to run
                        let a_minute = seeder_info.timestamp < chrono::Utc::now().timestamp()-60;
                        if seeder_info.timestamp != old_seeder_info.timestamp && !a_hour {
                            if &seeder_info.action[..] == "joinServer" {
                                // remove old session when switching to fast
                                if &old_seeder_info.game_id[..] != &seeder_info.game_id[..] && &old_seeder_info.action[..] == "joinServer" {
                                    quit_game();
                                }
                                launch_game(&cfg, &seeder_info);
                                // game state == running game
                                game_running.store(1, atomic::Ordering::Relaxed);
                            } else if &seeder_info.action[..] == "shutdownPC"  && cfg.allow_shutdown && !a_minute {
                                match shutdown() {
                                    Ok(_) => println!("Shutting down, bye!"),
                                    Err(error) => eprintln!("Failed to shut down: {}", error),
                                }
                            } else {
                                quit_game();
                                // game state == no game
                                game_running.store(0, atomic::Ordering::Relaxed);
                            }
                        } else if seeder_info.timestamp != old_seeder_info.timestamp && a_hour {
                            println!("request older than a hour, not running latest request.")
                        } else {
                            if !&game_info.is_running && &seeder_info.action[..] == "joinServer" && seeder_info.rejoin {
                                println!("didn't find game running, starting..");
                                launch_game(&cfg, &seeder_info);
                            }
                        }
                        ping_backend(&cfg, &game_info);
                        old_seeder_info = seeder_info.clone();
                    },
                    Err(e) => {
                        println!("Failed to get info about server to join: {}", e);
                        println!("reconnecting...");
                    }
                }
            },
            Err(e) => {
                println!("Failed to connect to backend: {}", e);
                println!("reconnecting...");
            },
        }
        sleep(Duration::from_secs(10));
    }
}

fn ping_backend(cfg: &SeederConfig, game_info: &GameInfo) {
    match ureq::post("https://manager-api.gametools.network/api/seederinfo").send_json(ureq::json!({
        "groupid": cfg.group_id,
        "isrunning": game_info.is_running,
        "hostname": cfg.hostname
    })) {
        Ok(_) => {},
        Err(_) => println!("Couln't send update of client to backend")
    }
}

fn is_running() -> GameInfo {
    unsafe {
        let window: Vec<u16> = OsStr::new("Battlefield™ 1")
            .encode_wide()
            .chain(once(0))
            .collect();
        let window_handle = FindWindowW(std::ptr::null_mut(), window.as_ptr());
        let no_game: *mut HWND__ = ptr::null_mut();
        GameInfo{ is_running: window_handle != no_game, game_process: window_handle }
    }
}

fn quit_game() {
    println!("Quitting old session..");
    let game_process = winproc::Process::from_name("bf1.exe");
    match game_process {
        Ok(mut process) => {
            match process.terminate(1) {
                Ok(_) => println!("closed the game"),
                Err(e) => {
                    println!("failed to close game (likely permissions): {}", e);
                }
            }
        }
        Err(_) => {
            println!("no game process found!");
        }
    }
}

fn launch_game(cfg: &SeederConfig, seeder_info: &CurrentServer) {
    let game_id = &seeder_info.game_id[..];
    println!("joining id: {}", game_id);
    match Command::new(cfg.game_location.clone())
        .args([
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
        ])
        .spawn()
    {
        Ok(_) => println!("game launched"),
        Err(e) => println!("failed to launch game: {}", e),
    }
}

unsafe fn create_input(key_code: u16, flags: u32) -> INPUT {
    let mut input = mem::zeroed::<INPUT>();
    input.type_ = INPUT_KEYBOARD;
    let mut ki = input.u.ki_mut();
    ki.wVk = key_code;
    ki.dwFlags = flags;
    input
}

unsafe fn key_down(key_code: u16) {
    let mut input = create_input(key_code, 0);
    SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
}

unsafe fn key_up(key_code: u16) {
    let mut input = create_input(key_code, KEYEVENTF_KEYUP);
    SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
}

unsafe fn key_enter(key_code: u16) {
    key_down(key_code);
    sleep(Duration::from_millis(154));
    key_up(key_code);
}
