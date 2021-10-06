use std::process::Command;
use std::{
    ffi::CString,
    mem,
    sync::{atomic, Arc},
    thread::{self, sleep},
    time::Duration,
};
use tungstenite::{connect, Message};
use url::Url;
use winapi::um::winuser::{
    FindWindowA, SendInput, SetForegroundWindow, ShowWindow, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP,
};
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
}

/// `SeederConfig` implements `Default`
impl ::std::default::Default for SeederConfig {
    fn default() -> Self {
        Self {
            group_id: "0fda8e4c-5be3-11eb-b1da-cd4ff7dab605".into(),
            game_location: "C:\\Program Files (x86)\\Origin Games\\Battlefield 1\\bf1.exe".into(),
        }
    }
}

// "ws://localhost:5051/ws/seeder?groupid={}"
// "ws://seeder.gametools.network:5252/ws/seeder?groupid={}"
fn main() {
    loop {
        match web_client() {
            Ok(_) => {}
            Err(e) => println!("{:#?}", e),
        };
    }
}

fn web_client() -> Result<(), &'static str> {
    let game_running = Arc::new(atomic::AtomicU32::new(0));
    let game_running_clone = Arc::clone(&game_running);
    // anti afk thread, runs when game is running
    thread::spawn(move || loop {
        if game_running_clone.load(atomic::Ordering::Relaxed) == 1 {
            let window_name = CString::new("Battlefield™ 1").unwrap();
            unsafe {
                let window_handle = FindWindowA(std::ptr::null_mut(), window_name.as_ptr());
                SetForegroundWindow(window_handle);
                ShowWindow(window_handle, 9);
                sleep(Duration::from_millis(1808));
                key_enter(0x45);
                sleep(Duration::from_millis(100));
                ShowWindow(window_handle, 6);
                sleep(Duration::from_millis(120000));
            }
        }
    });

    let cfg: SeederConfig = confy::load_path("config.txt").unwrap();
    confy::store_path("config.txt", cfg.clone()).unwrap();
    let connect_addr = format!(
        "ws://seeder.gametools.network:5252/ws/seeder?groupid={}",
        cfg.group_id
    );

    // let (mut socket, _response) =
    match connect(Url::parse(&connect_addr[..]).unwrap()) {
        Ok((mut socket, _response)) => {
            println!("WebSocket handshake has been successfully completed");
            println!("Connected to the server with group id: {}", cfg.group_id);
            println!("Using game in location: {}", cfg.game_location);
            loop {
                match socket.read_message() {
                    Ok(msg) => {
                        if matches!(msg.clone(), Message::Text(_string)) {
                            if matches!(msg.clone(), Message::Text(_string)) {
                                let deserialized: BroadcastMessage =
                                    serde_json::from_str(&msg.into_text().unwrap()[..]).unwrap();
                                if &deserialized.action[..] == "joinServer" {
                                    let game_id = &deserialized.gameid[..];
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
                                    // game state == running game
                                    game_running.store(1, atomic::Ordering::Relaxed);
                                } else {
                                    println!("Quitting game..");
                                    let game_process = winproc::Process::from_name("bf1.exe");
                                    match game_process {
                                        Ok(mut process) => {
                                            match process.terminate(1) {
                                                Ok(_) => println!("closed the game"),
                                                Err(e) => {
                                                    println!("failed to close game (likely permissions): {}", e)
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            println!("no game process found!");
                                        }
                                    }
                                    // game state == no game
                                    game_running.store(0, atomic::Ordering::Relaxed);
                                }
                            } else if matches!(msg.clone(), Message::Ping(_)) {
                                match socket.write_message(Message::Pong(msg.into_data())) {
                                    Ok(_) => {}
                                    Err(e) => println!("Failed to send pong: {}", e),
                                }
                            } else {
                                println!("{:#?}", msg.clone());
                            }
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                        return Err("Restarting...");
                    }
                }
            }
        }
        Err(e) => {
            println!("{}", e);
            return Err("Restarting...");
        }
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
