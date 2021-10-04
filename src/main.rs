use std::process::Command;

use futures_util::{future, pin_mut, StreamExt};
use tokio::io::AsyncReadExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
#[macro_use] extern crate serde_derive;

#[derive(Serialize, Deserialize, Debug)]
struct BroadcastMessage {
    r#type: String,
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


#[tokio::main]
async fn main() {
    let cfg: SeederConfig = confy::load_path("config.txt").unwrap();
    confy::store_path("config.txt", cfg.clone()).unwrap();
    let connect_addr = format!("ws://seeder.gametools.network:5252/ws/seeder?groupid={}", cfg.group_id);

    let url = url::Url::parse(&connect_addr).unwrap();

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);

    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap();
			if matches!(data.clone(), Message::Text(_string)) {
				let deserialized: BroadcastMessage = serde_json::from_str(&data.into_text().unwrap()[..]).unwrap();
				let game = cfg.game_location.clone();
                let game_id = &deserialized.gameid[..];
                println!("joining id: {}", game_id);
				let _command = Command::new(game).args([
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
                ]).spawn().unwrap();
			}
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}