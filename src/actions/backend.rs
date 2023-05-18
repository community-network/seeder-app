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
    match ureq::post(format!("{}/api/seederinfo", cfg.endpoint).as_str())
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
