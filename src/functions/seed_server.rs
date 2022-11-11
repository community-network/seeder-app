use std::sync::atomic::AtomicU32;
use std::sync::{atomic, Arc};
use system_shutdown::shutdown;
use system_shutdown::reboot;

use crate::actions;
use crate::structs;

pub fn start(
    seeder_info: structs::CurrentServer,
    old_seeder_info: &mut structs::CurrentServer,
    cfg: &structs::SeederConfig,
    game_running: &Arc<AtomicU32>,
    retry_launch: &Arc<AtomicU32>,
    message_running: &Arc<AtomicU32>,
) {
    let game_info = actions::game::is_running();
    let a_hour = seeder_info.timestamp < chrono::Utc::now().timestamp() - 3600;
    let a_minute = seeder_info.timestamp < chrono::Utc::now().timestamp() - 60;
    let mut current_game_id = &seeder_info.game_id[..];
    let mut kp_seeder = false;
    let mut old_game_id = &old_seeder_info.game_id[..];
    if seeder_info.keep_alive_seeders.contains_key(&cfg.hostname)
    {
        // seeder is being used in mutlialive
        kp_seeder = true;  
        current_game_id = &seeder_info.keep_alive_seeders[&cfg.hostname]["gameId"];
    }
    if old_seeder_info.keep_alive_seeders.contains_key(&cfg.hostname)
    {
        old_game_id = &old_seeder_info.keep_alive_seeders[&cfg.hostname]["gameId"];
    }
    if seeder_info.timestamp != old_seeder_info.timestamp && !a_hour {
        if kp_seeder {
            //if gameid is different then old game id, or seedername not present in old arr, leave current session and start new
            if game_info.is_running 
            && (old_game_id != current_game_id && &old_seeder_info.action[..]!="leaveServer") 
            || (message_running.load(atomic::Ordering::Relaxed) == 1)    
            {
                actions::game::quit(&game_running, &retry_launch);
                // message is not running while seeding
                message_running.store(0, atomic::Ordering::Relaxed);
            }
            if !game_info.is_running
            {
                actions::game::launch(cfg, current_game_id, "soldier", &game_running, &retry_launch, old_game_id);
            }
            game_running.store(1, atomic::Ordering::Relaxed);
        } else if &seeder_info.action[..] == "joinServer" {
            // remove old session when switching to fast
            if (old_game_id != current_game_id && &old_seeder_info.action[..]!="leaveServer")
            || (message_running.load(atomic::Ordering::Relaxed) == 1)
            {
                actions::game::quit(&game_running, &retry_launch);
                // message is not running while seeding
                message_running.store(0, atomic::Ordering::Relaxed);
            }
            actions::game::launch(cfg, current_game_id, "soldier", &game_running, &retry_launch, old_game_id);
            // game state == running game
            game_running.store(1, atomic::Ordering::Relaxed);
        } else if &seeder_info.action[..] == "restartOrigin" && !a_minute {
            if game_info.is_running
            {
                actions::game::quit(&game_running, &retry_launch);
            }
            actions::launchers::restart_launcher(cfg);
        } else if &seeder_info.action[..] == "shutdownPC" && cfg.allow_shutdown && !a_minute {
            match shutdown() {
                Ok(_) => println!("Shutting down, bye!"),
                Err(error) => eprintln!("Failed to shut down: {}", error),
            }
        } else if &seeder_info.action[..] == "rebootPC" && !a_minute {
            match reboot() {
                Ok(_) => println!("Rebooting ..."),
                Err(error) => eprintln!("Failed to reboot: {}", error),
            }
        } else if &seeder_info.action[..] == "broadcastMessage" && cfg.send_messages {
            println!("broadcasting message...");
            actions::game::send_message(&seeder_info.game_id);
        } else if &seeder_info.action[..] == "leaveServer" {
            actions::game::quit(&game_running, &retry_launch);
            // game state == no game
        }
    } else if seeder_info.timestamp != old_seeder_info.timestamp && a_hour {
        println!("request older than a hour, not running latest request.")
    } else {
        if !&game_info.is_running && ((&seeder_info.action[..] == "joinServer" && seeder_info.rejoin) 
            || kp_seeder)
        {
            println!("didn't find game running, starting..");
            actions::game::launch(cfg, current_game_id, "soldier", &game_running, &retry_launch, old_game_id);
        }
        //set retries 0
        if game_info.is_running
        {
            retry_launch.store(0, atomic::Ordering::Relaxed);
        }
    }
    let origin_info = actions::launchers::is_launcher_running(cfg);
    actions::backend::ping(cfg, &game_info, &origin_info, retry_launch);
    *old_seeder_info = seeder_info.clone();
}
