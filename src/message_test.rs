use std::sync::{atomic, Arc};

use winapi::um::winuser::CF_DIB;
mod actions;
mod functions;
mod input;
mod structs;

fn main() {
    let mut cfg = structs::SeederConfig {
        hostname: hostname::get().unwrap().into_string().unwrap(),
        group_id: "".into(),
        game_location: "".into(),
        link2ea_location: "".into(),
        allow_shutdown: false,
        send_messages: true,
        usable_client: true,
        fullscreen_anti_afk: true,
        message: "test".into(),
        message_server_name: "".into(),
        message_start_time_utc: "12:00".into(),
        message_stop_time_utc: "23:00".into(),
        message_timeout_mins: 2,
        game: structs::Games::from("bf1"),
        launcher: structs::Launchers::from("steam"),
        endpoint: "https://manager-api.gametools.network".into(),
        anti_afk_timeout_secs: 120,
        backend_check_timeout_secs: 10,
    };
    let message_running = Arc::new(atomic::AtomicU32::new(1));

    cfg.game_location = actions::game::find_game(&cfg);
    cfg.link2ea_location = actions::launchers::find_link2ea();
    let game_running = Arc::new(atomic::AtomicU32::new(0));
    let message_timeout = Arc::new(atomic::AtomicU32::new(10));
    let current_message_id = Arc::new(atomic::AtomicU32::new(0));

    functions::anti_afk::start(
        &cfg,
        &game_running,
        &message_running,
        &message_timeout,
        &current_message_id,
    )
}
