mod owo;

use std::time::Duration;

use owo::playermanager::PlayerManager;
use tokio::{self, main, time::sleep};

#[main]
async fn main() {
    let mut player_manager = PlayerManager::new().await.unwrap();

    // if this doesnt work on here it wont work on js
    player_manager.set_event_callback(Box::new(|event: String| {
        match event.as_str() {
            "SessionsChanged" => {
                dbg!("SessionsChanged");
                player_manager.update_sessions(None);
            }
            "CurrentSessionChanged" => {
                dbg!("CurrentSessionChanged");
                player_manager.update_system_session();
            }
            _ => {
                dbg!(event);
            }
        }
    }));

    loop {
        sleep(Duration::from_millis(100)).await;
        if let Some(session) = player_manager.get_active_session() {
            session.pause().await;
        } else {
            println!("No session 💀")
        }
    }
}
