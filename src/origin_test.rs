use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
mod actions;
mod functions;
mod input;
mod structs;

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let mut cfg = structs::SeederConfig {
        hostname: hostname::get().unwrap().into_string().unwrap(),
        group_id: "".into(),
        game_location: "".into(),
        link2ea_location: "".into(),
        allow_shutdown: false,
        send_messages: false,
        usable_client: true,
        fullscreen_anti_afk: true,
        message: "Join our discord, we are recruiting: ...".into(),
        message_server_name: "".into(),
        message_start_time_utc: "12:00".into(),
        message_stop_time_utc: "23:00".into(),
        message_timeout_mins: 8,
        game: structs::Games::from("bf1"),
        launcher: structs::Launchers::from("origin"),
        endpoint: "https://manager-api.gametools.network".into(),
        anti_afk_timeout_secs: 120,
        backend_check_timeout_secs: 10,
    };
    cfg.game_location = actions::game::find_game(&cfg);
    cfg.link2ea_location = actions::launchers::find_link2ea();

    actions::launchers::launch_game_origin(&cfg, "9340024970330", "soldier");
}
