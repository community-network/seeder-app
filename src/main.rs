use std::process::{Child, Command};

use tungstenite::{connect, Message};
use url::Url;
#[macro_use] extern crate serde_derive;

#[derive(Serialize, Deserialize, Debug)]
struct BroadcastMessage {
    action: String,
    gameid: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct SeederConfig {
    group_id: String,
    game_location: String,
}

/// `SeederConfig` implements `Default`
impl ::std::default::Default for SeederConfig {
    fn default() -> Self { Self { group_id: "0fda8e4c-5be3-11eb-b1da-cd4ff7dab605".into(), game_location: "C:\\Program Files (x86)\\Origin Games\\Battlefield 1\\bf1.exe".into() } }
}

fn main() {
    let mut game: Option<Child> = None;
    let cfg: SeederConfig = confy::load_path("config.txt").unwrap();
    confy::store_path("config.txt", cfg.clone()).unwrap();
    let connect_addr = format!("ws://seeder.gametools.network:5252/ws/seeder?groupid={}", cfg.group_id);
    let (mut socket, _response) =
        connect(Url::parse(&connect_addr[..]).unwrap()).expect("Can't connect");

    println!("Connected to the server with group id: {}", cfg.group_id);
    println!("Using game in location: {}", cfg.game_location);

    loop {
        let msg = socket.read_message().expect("Error reading message");
        if matches!(msg.clone(), Message::Text(_string)) {
            let deserialized: BroadcastMessage = serde_json::from_str(&msg.into_text().unwrap()[..]).unwrap();
            if &deserialized.action[..] == "joinServer" {
                let game_id = &deserialized.gameid[..];
                println!("joining id: {}", game_id);
                game = Some(Command::new(cfg.game_location.clone()).args([
                    "-webMode",
                    "MP",
                    "-Origin_NoAppFocus",
                    "--activate-webhelper",
                    "-requestState",
                    "State_ClaimReservation",
                    "-gameId",
                    game_id,
                    "-gameMode",
                    "MP",
                    "-role",
                    "soldier",
                    "-asSpectator",
                ]).spawn().unwrap());
            } else {
                println!("Quitting game..");
                match game {
                    Some(ref mut process) => process.kill().unwrap(),
                    None => println!("No game to quit!")
                }
            }
        }
    }
    // socket.close(None);
}