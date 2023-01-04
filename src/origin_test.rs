use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
mod actions;
mod functions;
mod input;
mod structs;

fn main() {
    Builder::new()
    .format(|buf, record| {
        writeln!(buf,
            "{} [{}] - {}",
            Local::now().format("%Y-%m-%dT%H:%M:%S"),
            record.level(),
            record.args()
        )
    })
    .filter(None, LevelFilter::Info)
    .init();

    let cfg = structs::SeederConfig {
        hostname: hostname::get().unwrap().into_string().unwrap(),
        group_id: "".into(),
        game_location: actions::game::find_game(),
        allow_shutdown: false,
        send_messages: false,
        usable_client: true,
        fullscreen_anti_afk: true,
        use_ea_desktop: true,
        message: "Join our discord, we are recruiting: ...".into(),
        message_server_name: "".into(),
        message_start_time_utc: "12:00".into(),
        message_stop_time_utc: "23:00".into(),
        message_timeout_mins: 8,
    };

    actions::launchers::launch_game_origin(&cfg, "7821536030132", "soldier");
}