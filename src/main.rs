use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::collections::HashMap;
use std::io::Write;
use std::{
    sync::{atomic, Arc},
    thread::{self, sleep},
    time::Duration,
};
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

    // game_running based on api, 0 == leaving servers. 1 means joining servers.
    let game_running = Arc::new(atomic::AtomicU32::new(0));
    let game_running_clone_anti_afk = Arc::clone(&game_running);
    let game_running_clone_message = Arc::clone(&game_running);

    let message_running = Arc::new(atomic::AtomicU32::new(0));
    let message_running_clone = Arc::clone(&message_running);
    let message_running_clone_anti_afk = Arc::clone(&message_running);

    let current_message_id = Arc::new(atomic::AtomicU32::new(0));

    let message_timeout = Arc::new(atomic::AtomicU32::new(0));

    let retry_launch = Arc::new(atomic::AtomicU32::new(0));
    let retry_launch_clone_message = Arc::clone(&retry_launch);
    // get/set config
    let cfg: structs::SeederConfig = match confy::load_path("config.txt") {
        Ok(config) => config,
        Err(e) => {
            log::error!("error in config.txt: {}", e);
            log::warn!("changing back to default..");
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
                launcher: structs::Launchers::from("ea_desktop"),
                endpoint: "https://manager-api.gametools.network".into(),
            };
            cfg.game_location = actions::game::find_game(&cfg);
            cfg.link2ea_location = actions::launchers::find_link2ea();
            cfg
        }
    };

    confy::store_path("config.txt", cfg.clone()).unwrap();
    if cfg.group_id.is_empty() {
        log::warn!("group_id isn't set!");
    }

    // anti afk thread, runs when game is in "joined" state
    let afk_cfg = cfg.clone();
    thread::spawn(move || loop {
        functions::anti_afk::start(
            &afk_cfg,
            &game_running_clone_anti_afk,
            &message_running_clone_anti_afk,
            &message_timeout,
            &current_message_id,
        )
    });

    // send messages in server thread
    let message_cfg = cfg.clone();
    thread::spawn(move || loop {
        functions::auto_message::start(
            &game_running_clone_message,
            &retry_launch_clone_message,
            &message_cfg,
            &message_running,
        );
    });

    // do seeding
    let mut old_seeder_info = structs::CurrentServer {
        game_id: "".into(),
        action: "leaveServer".into(),
        group_id: cfg.group_id.clone(),
        timestamp: chrono::Utc::now().timestamp(),
        keep_alive_seeders: HashMap::new(),
        seeder_arr: vec![],
        rejoin: true,
    };
    let connect_addr = format!(
        "{}/api/getseeder?groupid={}&game={}",
        cfg.endpoint,
        cfg.group_id,
        cfg.game.short_name()
    );
    log::info!("firing of latest request found (default on startup script)");
    loop {
        match ureq::get(&connect_addr[..])
            .timeout(Duration::new(10, 0))
            .call()
        {
            Ok(response) => match response.header("type") {
                Some(_) => match response.into_json::<structs::Error>() {
                    Ok(error_return) => {
                        log::error!("Error: {}", error_return.error);
                        log::info!("retrying...");
                    }
                    Err(e) => {
                        log::error!("Incorrect error return: {}", e);
                        log::info!("retrying...");
                    }
                },
                None => match response.into_json::<structs::CurrentServer>() {
                    Ok(seeder_info) => {
                        functions::seed_server::start(
                            seeder_info,
                            &mut old_seeder_info,
                            &cfg,
                            &game_running,
                            &retry_launch,
                            &message_running_clone,
                        );
                    }
                    Err(e) => {
                        log::error!("Failed to get info about server to join: {}", e);
                        log::info!("reconnecting...");
                    }
                },
            },
            Err(e) => {
                log::error!("Failed to connect to backend: {}", e);
                log::info!("reconnecting...");
            }
        }
        sleep(Duration::from_secs(10));
    }
}
