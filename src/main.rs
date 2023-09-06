mod player;
mod playermanager;
mod types;
mod util;
use std::time::Duration;

use playermanager::PlayerManager;
use tokio::{self, main, time::sleep};

#[main]
async fn main() {
    let player_manager = PlayerManager::new(None).await.unwrap();
    loop {
        sleep(Duration::from_millis(100)).await;
        if let Some(session) = player_manager.lock().await.get_active_session() {
            session.pause().await;
        } else {
            println!("No session 💀")
        }
    }
}
