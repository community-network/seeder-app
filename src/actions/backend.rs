use crate::structs;
use std::sync::atomic::AtomicU32;
use std::sync::{atomic, Arc};
use std::time::Duration;

pub fn ping(
    cfg: &structs::SeederConfig,
    game_info: &structs::GameInfo,
    origin_info: &structs::GameInfo,
    retry_launch: &Arc<AtomicU32>,
) {
    match ureq::post("https://manager-api.gametools.network/api/seederinfo")
        .timeout(Duration::new(10, 0))
        .send_json(ureq::json!({
            "groupid": cfg.group_id,
            "isrunning": game_info.is_running,
            "retrycount": retry_launch.load(atomic::Ordering::Relaxed),
            "hostname": cfg.hostname,
            "isoriginrunning": origin_info.is_running,
            "game": cfg.game.short_name(),
        })) {
        Ok(_) => {}
        Err(_) => log::error!("Couln't send update of client to backend"),
    }
}

pub fn has_player(cfg: &structs::SeederConfig, game_id: &str) -> bool {
    if cfg.game == structs::Games::Bf4 {
        return bf4_has_player(cfg, game_id);
    }
    bf1_has_player(cfg, game_id)
}

pub fn bf1_has_player(cfg: &structs::SeederConfig, game_id: &str) -> bool {
    let url = format!(
        "https://api.gametools.network/bf1/players/?gameid={}",
        game_id
    );
    match ureq::get(&url[..]).call() {
        Ok(response) => match response.into_json::<structs::GametoolsPlayers>() {
            Ok(server_info) => {
                // if valid timestamp
                match chrono::NaiveDateTime::from_timestamp_millis(
                    server_info.update_timestamp * 1000,
                ) {
                    Some(naive_time) => {
                        let timestamp_time =
                            chrono::DateTime::<chrono::Utc>::from_utc(naive_time, chrono::Utc)
                                .time();
                        let current = chrono::Utc::now().time();
                        let diff = current - timestamp_time;
                        if diff.num_minutes() > 2 {
                            return true;
                        }

                        if !cfg.seeder_name.is_empty()
                            && (server_info.teams[0].players.contains(
                                &structs::GametoolsServerPlayer {
                                    name: cfg.seeder_name.clone(),
                                },
                            ) || server_info.teams[1].players.contains(
                                &structs::GametoolsServerPlayer {
                                    name: cfg.seeder_name.clone(),
                                },
                            ))
                        {
                            return true;
                        }
                        false
                    }
                    None => {
                        log::error!("no timestamp in players?");
                        log::info!("reconnecting...");
                        true
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get info about server to join: {}", e);
                log::info!("reconnecting...");
                true
            }
        },
        Err(e) => {
            log::error!("Failed to connect to gametools: {}", e);
            log::info!("reconnecting...");
            true
        }
    }
}

pub fn bf4_has_player(cfg: &structs::SeederConfig, game_id: &str) -> bool {
    let url = format!(
        "https://api.gametools.network/bf4/detailedserver/?gameid={}&lang=en-us&region=all&platform=pc&service=undefined&",
        game_id
    );
    match ureq::get(&url[..]).call() {
        Ok(response) => match response.into_json::<structs::GametoolsDetailedServer>() {
            Ok(server_info) => {
                if !cfg.seeder_name.is_empty()
                    && !server_info
                        .players
                        .unwrap()
                        .contains(&structs::GametoolsServerPlayer {
                            name: cfg.seeder_name.clone(),
                        })
                {
                    return true;
                }
                false
            }
            Err(e) => {
                log::error!("Failed to get info about server to join: {}", e);
                log::info!("reconnecting...");
                true
            }
        },
        Err(e) => {
            log::error!("Failed to connect to gametools: {}", e);
            log::info!("reconnecting...");
            true
        }
    }
}
