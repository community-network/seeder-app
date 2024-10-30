use crate::actions;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use winapi::shared::windef::HWND__;

#[derive(Serialize, Deserialize, Debug)]
pub struct BroadcastMessage {
    pub action: String,
    pub gameid: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SeederConfig {
    pub group_id: String,
    pub game_location: String,
    pub link2ea_location: String,
    pub hostname: String,
    pub allow_shutdown: bool,
    // when i'ts done seeding, join for messages
    pub send_messages: bool,
    pub usable_client: bool,
    pub fullscreen_anti_afk: bool,
    pub message: String,
    pub message_server_name: String,
    pub message_start_time_utc: String,
    pub message_stop_time_utc: String,
    pub message_timeout_mins: u32,
    pub game: Games,
    pub launcher: Launchers,
    pub endpoint: String,
    pub anti_afk_timeout_secs: u64,
    pub backend_check_timeout_secs: u64,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct CurrentServer {
    #[serde(rename = "gameId")]
    pub game_id: String,
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "timeStamp")]
    pub timestamp: i64,
    pub action: String,
    #[serde(rename = "keepAliveSeeders")]
    pub keep_alive_seeders: HashMap<String, HashMap<String, String>>,
    #[serde(rename = "seederArr")]
    pub seeder_arr: Vec<String>,
    pub rejoin: bool,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Error {
    pub error: String,
}

#[derive(Debug)]
pub struct GameInfo {
    pub is_running: bool,
    pub game_process: *mut HWND__,
    pub anticheat_launcher_running: bool,
}

/// `SeederConfig` implements `Default`
impl ::std::default::Default for SeederConfig {
    fn default() -> Self {
        let mut cfg = Self {
            hostname: hostname::get().unwrap().into_string().unwrap(),
            group_id: "".into(),
            game_location: "".into(),
            link2ea_location: "".into(),
            allow_shutdown: false,
            send_messages: false,
            usable_client: true,
            fullscreen_anti_afk: true,
            message: "Join our discord, we are recruiting: discord.gg/BoB".into(),
            message_server_name: "[BoB]#1 EU".into(),
            message_start_time_utc: "12:00".into(),
            message_stop_time_utc: "23:00".into(),
            message_timeout_mins: 8,
            game: Games::from("bf1"),
            launcher: Launchers::from("ea_desktop"),
            endpoint: "https://manager-api.gametools.network".into(),
            anti_afk_timeout_secs: 120,
            backend_check_timeout_secs: 10,
        };
        cfg.game_location = actions::game::find_game(&cfg);
        cfg.link2ea_location = actions::launchers::find_link2ea();
        cfg
    }
}

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct ServerList {
    pub servers: Vec<ServerInfo>,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct ServerInfo {
    #[serde(rename = "gameId")]
    pub game_id: String,
}

#[derive(Clone, Debug)]
pub struct EaDesktopNewestFile {
    pub file_name: String,
    pub time: u64,
    pub location: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Launchers {
    EADesktop,
    Origin,
    Steam,
}

impl Launchers {
    pub fn from(input: &str) -> Launchers {
        match input {
            "ea_desktop" => Launchers::EADesktop,
            "origin" => Launchers::Origin,
            "steam" => Launchers::Steam,
            _ => Launchers::EADesktop,
        }
    }

    pub fn window_name(&self) -> &'static str {
        match self {
            Launchers::EADesktop => "EA",
            Launchers::Origin => "Origin",
            Launchers::Steam => "Steam",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Games {
    Bf4,
    Bf1,
}

impl Games {
    pub fn from(input: &str) -> Games {
        match input {
            "bf4" => Games::Bf4,
            "bf1" => Games::Bf1,
            _ => Games::Bf1,
        }
    }

    pub fn full_name(&self) -> &'static str {
        match self {
            Games::Bf4 => "Battlefield 4",
            Games::Bf1 => "Battlefield 1",
        }
    }

    pub fn window_name(&self) -> &'static str {
        match self {
            Games::Bf4 => "Battlefield 4",
            Games::Bf1 => "Battlefield™ 1",
        }
    }

    pub fn process_name(&self) -> &'static str {
        match self {
            Games::Bf4 => "bf4.exe",
            Games::Bf1 => "bf1.exe",
        }
    }

    pub fn process_start(&self) -> &'static str {
        match self {
            Games::Bf4 => "BFLauncher_x86.exe",
            Games::Bf1 => "bf1.exe",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Games::Bf4 => "bf4",
            Games::Bf1 => "bf1",
        }
    }

    pub fn game_versions(&self) -> Vec<&'static str> {
        match self {
            Games::Bf4 => vec![
                "user.gamecommandline.origin.ofr.50.0002683",
                "user.gamecommandline.ofb-east:109552316",
                "user.gamecommandline.ofb-east:109546867",
                "user.gamecommandline.ofb-east:109549060",
            ],
            Games::Bf1 => vec![
                "user.gamecommandline.origin.ofr.50.0000557",
                "user.gamecommandline.origin.ofr.50.0001382",
                "user.gamecommandline.origin.ofr.50.0001665",
                "user.gamecommandline.origin.ofr.50.0001662",
                "user.gamecommandline.origin.ofr.50.0001390",
            ],
        }
    }
}
