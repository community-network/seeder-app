use chrono::{NaiveTime, Utc};
use std::sync::atomic::AtomicU32;
use std::{
    sync::{atomic, Arc},
    thread::sleep,
    time::Duration,
};
use system_shutdown::shutdown;
use urlencoding::encode;
use winapi::um::winuser::{SetForegroundWindow, ShowWindow};

use crate::actions;
use crate::chars::{DXCode, char_to_dxcodes};
use crate::send_keys;
use crate::structs;

pub fn anti_afk(cfg: &structs::SeederConfig, game_running: &Arc<AtomicU32>, message_running: &Arc<AtomicU32>) {
    // run when seeding or message
    if game_running.load(atomic::Ordering::Relaxed) == 1 {
        let game_info = actions::is_running();
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
    if message_running.load(atomic::Ordering::Relaxed) == 1 {
        let game_info = actions::is_running();
        if game_info.is_running {
            unsafe {
                SetForegroundWindow(game_info.game_process);
                ShowWindow(game_info.game_process, 9);
                sleep(Duration::from_millis(1808));
                send_keys::key_enter(0x24, 8);
                sleep(Duration::from_millis(800));
                let mut message: Vec<DXCode> = Vec::new();
                for char in cfg.message.chars() {
                    match char_to_dxcodes(char) {
                        Some(dx) => message.push(dx),
                        None => {},
                    }
                }
                send_keys::send_string(message);
                sleep(Duration::from_millis(100));
                send_keys::key_enter(0x1C, 8);
                sleep(Duration::from_millis(100));
                ShowWindow(game_info.game_process, 6);
            }
        }
    }
    sleep(Duration::from_secs(120));
}

pub fn auto_message(
    game_running: &Arc<atomic::AtomicU32>,
    cfg: &structs::SeederConfig,
    message_running: &Arc<atomic::AtomicU32>,
) {
    // only run when not seeding
    sleep(Duration::from_secs(10));
    if (game_running.load(atomic::Ordering::Relaxed) == 0) && cfg.send_messages {
        let start_time = &mut cfg.message_start_time_utc.split(':');
        let stop_time = &mut cfg.message_stop_time_utc.split(':');
        let low = NaiveTime::from_hms(
            start_time
                .next()
                .unwrap_or("12")
                .parse::<u32>()
                .unwrap_or(12),
            start_time
                .next()
                .unwrap_or("00")
                .parse::<u32>()
                .unwrap_or(0),
            0,
        );
        let high = NaiveTime::from_hms(
            stop_time
                .next()
                .unwrap_or("23")
                .parse::<u32>()
                .unwrap_or(23),
            stop_time.next().unwrap_or("00").parse::<u32>().unwrap_or(0),
            0,
        );
        let time_of_day = Utc::now().time();
        if (time_of_day > low) && (time_of_day < high) {
            message_running.store(1, atomic::Ordering::Relaxed);
            let game_info = actions::is_running();
            if !&game_info.is_running {
                println!("didn't find game running for message, starting..");
                let connect_addr = format!(
                    "https://api.gametools.network/bf1/servers/?name={}&region=all&platform=pc&limit=1&lang=en-us",
                    encode(&cfg.message_server_name[..])
                );
                match ureq::get(&connect_addr[..]).call() {
                    Ok(response) => match response.into_json::<structs::ServerList>() {
                        Ok(server_info) => {
                            actions::launch_game(cfg, &server_info.servers[0].game_id, "spectator");
                        }
                        Err(_) => println!("Servername not found"),
                    },
                    Err(e) => {
                        println!("Failed to connect to Main API: {}", e);
                        println!("retrying...")
                    }
                }
            }
        } else {
            message_running.store(0, atomic::Ordering::Relaxed);
        }
    }
}

pub fn seed_server(
    seeder_info: structs::CurrentServer,
    old_seeder_info: &mut structs::CurrentServer,
    cfg: &structs::SeederConfig,
    game_running: &Arc<AtomicU32>,
    message_running: &Arc<AtomicU32>,
) {
    let game_info = actions::is_running();
    let a_hour = seeder_info.timestamp < chrono::Utc::now().timestamp() - 3600;
    let a_minute = seeder_info.timestamp < chrono::Utc::now().timestamp() - 60;
    if seeder_info.timestamp != old_seeder_info.timestamp && !a_hour {
        if &seeder_info.action[..] == "joinServer" {
            // remove old session when switching to fast
            if (&old_seeder_info.game_id[..] != &seeder_info.game_id[..]
                && &old_seeder_info.action[..] == "joinServer")
                || (message_running.load(atomic::Ordering::Relaxed) == 1)
            {
                actions::quit_game();
                // message is not running while seeding
                message_running.store(0, atomic::Ordering::Relaxed);
            }
            actions::launch_game(cfg, &seeder_info.game_id[..], "soldier");
            // game state == running game
            game_running.store(1, atomic::Ordering::Relaxed);
        } else if &seeder_info.action[..] == "shutdownPC" && cfg.allow_shutdown && !a_minute {
            match shutdown() {
                Ok(_) => println!("Shutting down, bye!"),
                Err(error) => eprintln!("Failed to shut down: {}", error),
            }
        } else {
            // actions::quit_game();
            // game state == no game
            game_running.store(0, atomic::Ordering::Relaxed);
        }
    } else if seeder_info.timestamp != old_seeder_info.timestamp && a_hour {
        println!("request older than a hour, not running latest request.")
    } else {
        if !&game_info.is_running && &seeder_info.action[..] == "joinServer" && seeder_info.rejoin {
            println!("didn't find game running, starting..");
            actions::launch_game(cfg, &seeder_info.game_id[..], "soldier");
        }
    }
    actions::ping_backend(cfg, &game_info);
    *old_seeder_info = seeder_info.clone();
}
