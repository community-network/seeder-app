use chrono::{NaiveTime, Utc};
use std::{
    sync::{atomic, Arc},
    thread::sleep,
    time::Duration,
};
use urlencoding::encode;

use crate::actions;
use crate::structs;

pub fn start(
    game_running: &Arc<atomic::AtomicU32>,
    retry_launch: &Arc<atomic::AtomicU32>,
    cfg: &structs::SeederConfig,
    message_running: &Arc<atomic::AtomicU32>,
) {
    // only run when not seeding
    sleep(Duration::from_secs(10));
    if (game_running.load(atomic::Ordering::Relaxed) == 0) && cfg.send_messages {
        let start_time = &mut cfg.message_start_time_utc.split(':');
        let stop_time = &mut cfg.message_stop_time_utc.split(':');
        let low = match NaiveTime::from_hms_opt(
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
        ) {
            Some(low) => low,
            None => return log::error!("Failed to create time object from start time"),
        };
        let high = match NaiveTime::from_hms_opt(
            stop_time
                .next()
                .unwrap_or("23")
                .parse::<u32>()
                .unwrap_or(23),
            stop_time.next().unwrap_or("00").parse::<u32>().unwrap_or(0),
            0,
        ) {
            Some(high) => high,
            None => return log::error!("Failed to create time object from stop time"),
        };
        let time_of_day = Utc::now().time();
        if (time_of_day > low) && (time_of_day < high) {
            message_running.store(1, atomic::Ordering::Relaxed);
            let game_info = actions::game::is_running();
            if !&game_info.is_running {
                log::warn!("didn't find game running for message, starting..");
                let connect_addr = format!(
                    "https://api.gametools.network/bf1/servers/?name={}&region=all&platform=pc&limit=1&lang=en-us",
                    encode(&cfg.message_server_name[..])
                );
                match ureq::get(&connect_addr[..]).timeout(Duration::new(10, 0)).call() {
                    Ok(response) => match response.into_json::<structs::ServerList>() {
                        Ok(server_info) => {
                            actions::game::launch(cfg, &server_info.servers[0].game_id, "spectator", &game_running, &retry_launch);
                        }
                        Err(_) => log::error!("Servername not found"),
                    },
                    Err(e) => {
                        log::error!("Failed to connect to Main API: {}", e);
                        log::info!("retrying...")
                    }
                }
            }
        } else {
            message_running.store(0, atomic::Ordering::Relaxed);
        }
    }
}