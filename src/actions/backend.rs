use std::time::Duration;
use std::sync::{atomic, Arc};
use std::sync::atomic::AtomicU32;
use crate::structs;

pub fn ping(cfg: &structs::SeederConfig, game_info: &structs::GameInfo, origin_info: &structs::GameInfo, retry_launch: &Arc<AtomicU32>) {
    match ureq::post("https://manager-api.gametools.network/api/seederinfo").timeout(Duration::new(10, 0)).send_json(
        ureq::json!({
            "groupid": cfg.group_id,
            "isrunning": game_info.is_running,
            "retrycount": retry_launch.load(atomic::Ordering::Relaxed),
            "hostname": cfg.hostname,
            "isoriginrunning": origin_info.is_running
        }),
    ) {
        Ok(_) => {}
        Err(_) => println!("Couln't send update of client to backend"),
    }
}
