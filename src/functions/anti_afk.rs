use std::convert::TryInto;
use std::sync::atomic::AtomicU32;
use std::{
    sync::{atomic, Arc},
    thread::sleep,
    time::Duration,
};

use crate::actions;
use crate::structs;

pub fn start(
    cfg: &structs::SeederConfig,
    game_running: &Arc<AtomicU32>,
    message_running: &Arc<AtomicU32>,
    message_timeout: &Arc<AtomicU32>,
    current_message_id: &Arc<AtomicU32>,
) {
    // run when seeding or message
    if game_running.load(atomic::Ordering::Relaxed) == 1 {
        let fullscreen = actions::game::is_fullscreen(cfg);
        if !fullscreen || (fullscreen && cfg.fullscreen_anti_afk) {
            actions::game::anti_afk(cfg);
        }
    }
    if message_running.load(atomic::Ordering::Relaxed) == 1 {
        let timeout = message_timeout.load(atomic::Ordering::Relaxed);
        if timeout >= (cfg.message_timeout_mins / 2) {
            // split message with ";" and send different one each time
            let mut message_id = current_message_id.load(atomic::Ordering::Relaxed);
            let split = cfg.message.split(';');

            let current_message: Vec<&str> = split.clone().collect();
            let message: &str = current_message[message_id as usize];

            // send message
            log::info!("{}", message);
            log::info!("sending message...");
            actions::game::send_message(message.to_string(), cfg);
            message_timeout.store(0, atomic::Ordering::Relaxed);

            // next message in list, 0 if no new items in ";" split
            if message_id + 1 >= split.count().try_into().unwrap() {
                message_id = 0;
            } else {
                message_id += 1;
            }

            // save
            current_message_id.store(message_id, atomic::Ordering::Relaxed);
        } else {
            actions::game::anti_afk(cfg);
            message_timeout.store(timeout + 1, atomic::Ordering::Relaxed);
        }
    }
    sleep(Duration::from_secs(cfg.anti_afk_timeout_secs));
}
