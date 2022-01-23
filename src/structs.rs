use serde_derive::{Deserialize, Serialize};
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
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct CurrentServer {
    #[serde(rename = "gameId")]
    pub game_id: String,
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "timeStamp")]
    pub timestamp: i64,
    pub action: String,
    pub rejoin: bool,
}

pub struct GameInfo {
    pub is_running: bool,
    pub game_process: *mut HWND__,
}

/// `SeederConfig` implements `Default`
impl ::std::default::Default for SeederConfig {
    fn default() -> Self {
        Self {
            hostname: hostname::get().unwrap().into_string().unwrap(),
            group_id: "0fda8e4c-5be3-11eb-b1da-cd4ff7dab605".into(),
            game_location: "C:\\Program Files (x86)\\Origin Games\\Battlefield 1\\bf1.exe".into(),
            allow_shutdown: false,
            send_messages: false,
            usable_client: true,
            fullscreen_anti_afk: true,
            message: "Join our discord, we are recruiting: discord.gg/BoB".into(),
            message_server_name: "[BoB]#1 EU".into(),
            message_start_time_utc: "12:00".into(),
            message_stop_time_utc: "23:00".into(),
            message_timeout_mins: 8,
        }
    }
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct ServerList {
    pub servers: Vec<ServerInfo>,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct ServerInfo {
    #[serde(rename = "gameId")]
    pub game_id: String,
}