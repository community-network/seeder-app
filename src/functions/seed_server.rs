use std::sync::atomic::AtomicU32;
use std::sync::{atomic, Arc};
use system_shutdown::reboot;
use system_shutdown::shutdown;

use crate::actions;
use crate::structs;

fn multialive(
    cfg: &structs::SeederConfig,
    game_info: &structs::GameInfo,
    old_seeder_info: &structs::CurrentServer,
    current_game_id: &str,
    old_game_id: &str,
    game_running: &Arc<AtomicU32>,
    retry_launch: &Arc<AtomicU32>,
    message_running: &Arc<AtomicU32>,
) {
    //if gameid is different then old game id, or seedername not present in old arr, leave current session and start new
    if game_info.is_running
        && (old_game_id != current_game_id && &old_seeder_info.action[..] != "leaveServer")
        || (message_running.load(atomic::Ordering::Relaxed) == 1)
    {
        actions::game::quit(cfg, game_running, retry_launch);
        // message is not running while seeding
        message_running.store(0, atomic::Ordering::Relaxed);
    }
    if !game_info.is_running {
        actions::game::launch(cfg, current_game_id, "soldier", game_running, retry_launch);
    }
    game_running.store(1, atomic::Ordering::Relaxed);
}

fn on_command_changed(
    cfg: &structs::SeederConfig,
    kp_seeder: bool,
    game_info: &structs::GameInfo,
    seeder_info: &structs::CurrentServer,
    old_seeder_info: &structs::CurrentServer,
    current_game_id: &str,
    old_game_id: &str,
    game_running: &Arc<AtomicU32>,
    retry_launch: &Arc<AtomicU32>,
    message_running: &Arc<AtomicU32>,
) {
    let a_minute = seeder_info.timestamp < chrono::Utc::now().timestamp() - 60; // 1 minute since last request

    if kp_seeder {
        multialive(
            cfg,
            game_info,
            old_seeder_info,
            current_game_id,
            old_game_id,
            game_running,
            retry_launch,
            message_running,
        );
    } else if &seeder_info.action[..] == "joinServer" {
        // remove old session when switching to fast
        if (old_game_id != current_game_id && &old_seeder_info.action[..] != "leaveServer")
            || (message_running.load(atomic::Ordering::Relaxed) == 1)
        {
            actions::game::quit(cfg, game_running, retry_launch);
            // message is not running while seeding
            message_running.store(0, atomic::Ordering::Relaxed);
        }
        // for ea desktop, only if game is not running already
        if !game_info.is_running {
            actions::game::launch(cfg, current_game_id, "soldier", game_running, retry_launch);
        }
        // game state == running game
        game_running.store(1, atomic::Ordering::Relaxed);
    } else if &seeder_info.action[..] == "restartOrigin" && !a_minute {
        if game_info.is_running {
            actions::game::quit(cfg, game_running, retry_launch);
        }
        actions::launchers::restart_launcher(cfg);
    } else if &seeder_info.action[..] == "shutdownPC" && cfg.allow_shutdown && !a_minute {
        match shutdown() {
            Ok(_) => log::info!("Shutting down, bye!"),
            Err(error) => log::error!("Failed to shut down: {}", error),
        }
    } else if &seeder_info.action[..] == "rebootPC" && !a_minute {
        match reboot() {
            Ok(_) => log::info!("Rebooting ..."),
            Err(error) => log::error!("Failed to reboot: {}", error),
        }
    } else if &seeder_info.action[..] == "broadcastMessage" && cfg.send_messages {
        log::info!("broadcasting message...");
        actions::game::send_message(seeder_info.game_id.clone(), cfg);
    } else if &seeder_info.action[..] == "leaveServer" {
        actions::game::quit(cfg, game_running, retry_launch);
        // game state == no game
    }
}

fn retry_check(
    cfg: &structs::SeederConfig,
    kp_seeder: bool,
    game_info: &structs::GameInfo,
    seeder_info: &structs::CurrentServer,
    current_game_id: &str,
    game_running: &Arc<AtomicU32>,
    retry_launch: &Arc<AtomicU32>,
    retry_player_check: &Arc<AtomicU32>,
) {
    // if game isnt running but should: retry
    if !&game_info.is_running
        && ((&seeder_info.action[..] == "joinServer" && seeder_info.rejoin) || kp_seeder)
    {
        log::warn!("didn't find game running, starting..");
        actions::game::launch(cfg, current_game_id, "soldier", game_running, retry_launch);
    }
    // if game is running, check if in right server if option set
    if game_info.is_running {
        retry_launch.store(0, atomic::Ordering::Relaxed);

        // check if player is in server
        if !cfg.seeder_name.is_empty() {
            let retries = retry_player_check.load(atomic::Ordering::Relaxed);
            if actions::backend::has_player(cfg, current_game_id) {
                if retries > 0 {
                    log::info!("player found");
                }
                retry_player_check.store(0, atomic::Ordering::Relaxed)
            } else if retries >= cfg.find_player_max_retries {
                log::error!(
                    "player is still not in the server after {} retries",
                    retries
                );
                actions::game::quit(cfg, game_running, retry_launch);
                actions::game::launch(cfg, current_game_id, "soldier", game_running, retry_launch);
                retry_player_check.store(0, atomic::Ordering::Relaxed);
            } else {
                retry_player_check.fetch_add(1, atomic::Ordering::Relaxed);
                log::info!("player not yet found, try number: {}", retries);
            }
        }
    }
}

pub fn start(
    seeder_info: structs::CurrentServer,
    old_seeder_info: &mut structs::CurrentServer,
    cfg: &structs::SeederConfig,
    game_running: &Arc<AtomicU32>,
    retry_launch: &Arc<AtomicU32>,
    message_running: &Arc<AtomicU32>,
    retry_player_check: &Arc<AtomicU32>,
) {
    let game_info = actions::game::is_running(cfg);
    let a_hour = seeder_info.timestamp < chrono::Utc::now().timestamp() - 3600; // 1 hour since last request
    let mut current_game_id = &seeder_info.game_id[..]; // current server it has to join
    let mut kp_seeder = false; // is multialive seeder bool
    let mut old_game_id = &old_seeder_info.game_id[..]; // previous check (if changed, will be used to switch server)

    // set keepalive info when being used in multilalive
    if seeder_info.keep_alive_seeders.contains_key(&cfg.hostname) {
        kp_seeder = true;
        current_game_id = &seeder_info.keep_alive_seeders[&cfg.hostname]["gameId"];
    }
    if old_seeder_info
        .keep_alive_seeders
        .contains_key(&cfg.hostname)
    {
        old_game_id = &old_seeder_info.keep_alive_seeders[&cfg.hostname]["gameId"];
    }

    // Actions when the seeding command changed
    if seeder_info.timestamp != old_seeder_info.timestamp && !a_hour {
        on_command_changed(
            cfg,
            kp_seeder,
            &game_info,
            &seeder_info,
            old_seeder_info,
            current_game_id,
            old_game_id,
            game_running,
            retry_launch,
            message_running,
        );
    // request to old to work with
    } else if seeder_info.timestamp != old_seeder_info.timestamp && a_hour {
        log::info!("request older than a hour, not running latest request.")
    // if no new action
    } else {
        retry_check(
            cfg,
            kp_seeder,
            &game_info,
            &seeder_info,
            current_game_id,
            game_running,
            retry_launch,
            retry_player_check,
        );
    }

    // ping backend
    let origin_info = actions::launchers::is_launcher_running(cfg);
    actions::backend::ping(cfg, &game_info, &origin_info, retry_launch);
    *old_seeder_info = seeder_info.clone();
}
