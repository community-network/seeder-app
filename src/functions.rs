use chrono::{NaiveTime, Utc};
use std::sync::atomic::AtomicU32;
use std::{
    sync::{atomic, Arc},
    thread::sleep,
    time::Duration,
};
use system_shutdown::shutdown;
use winapi::um::winuser::{SetForegroundWindow, ShowWindow};

use crate::actions;
use crate::send_keys;
use crate::structs;

pub fn anti_afk(game_running: &Arc<AtomicU32>, message_running: &Arc<AtomicU32>) {
    // run when seeding or message
    if (game_running.load(atomic::Ordering::Relaxed) == 1)
        || (message_running.load(atomic::Ordering::Relaxed) == 1)
    {
        let game_info = actions::is_running();
        if game_info.is_running {
            unsafe {
                // if game is not running
                SetForegroundWindow(game_info.game_process);
                ShowWindow(game_info.game_process, 9);
                sleep(Duration::from_millis(1808));
                send_keys::key_enter(0x45);
                sleep(Duration::from_millis(100));
                ShowWindow(game_info.game_process, 6);
            }
        }
    }
    sleep(Duration::from_secs(120));
}

pub fn auto_message(
    game_running_clone_message: &Arc<atomic::AtomicU32>,
    cfg: &structs::SeederConfig,
    message_running: &Arc<atomic::AtomicU32>,
) {
    // only run when not seeding
    if game_running_clone_message.load(atomic::Ordering::Relaxed) == 0 {
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
            //TODO: send actual message
            let game_info = actions::is_running();
            if !&game_info.is_running {
                println!("didn't find game running for message, starting..");
                //TODO: get server gameid from name
                // actions::launch_game(cfg, &game_id);
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
    message_running_clone: &Arc<AtomicU32>,
) {
    let game_info = actions::is_running();
    let a_hour = seeder_info.timestamp < chrono::Utc::now().timestamp() - 3600;
    let a_minute = seeder_info.timestamp < chrono::Utc::now().timestamp() - 60;
    if seeder_info.timestamp != old_seeder_info.timestamp && !a_hour {
        if &seeder_info.action[..] == "joinServer" {
            // remove old session when switching to fast
            if (&old_seeder_info.game_id[..] != &seeder_info.game_id[..]
                && &old_seeder_info.action[..] == "joinServer")
                || (message_running_clone.load(atomic::Ordering::Relaxed) == 1)
            {
                actions::quit_game();
                // message is not running while seeding
                message_running_clone.store(0, atomic::Ordering::Relaxed);
            }
            actions::launch_game(cfg, &seeder_info.game_id[..]);
            // game state == running game
            game_running.store(1, atomic::Ordering::Relaxed);
        } else if &seeder_info.action[..] == "shutdownPC" && cfg.allow_shutdown && !a_minute {
            match shutdown() {
                Ok(_) => println!("Shutting down, bye!"),
                Err(error) => eprintln!("Failed to shut down: {}", error),
            }
        } else {
            actions::quit_game();
            // game state == no game
            game_running.store(0, atomic::Ordering::Relaxed);
        }
    } else if seeder_info.timestamp != old_seeder_info.timestamp && a_hour {
        println!("request older than a hour, not running latest request.")
    } else {
        if !&game_info.is_running && &seeder_info.action[..] == "joinServer" && seeder_info.rejoin {
            println!("didn't find game running, starting..");
            actions::launch_game(cfg, &seeder_info.game_id[..]);
        }
    }
    actions::ping_backend(cfg, &game_info);
    *old_seeder_info = seeder_info.clone();
}
