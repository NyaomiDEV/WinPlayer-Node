mod owo;

use std::time::Duration;

use owo::playermanager::{ManagerEvent, PlayerManager};
use tokio::{self, main, time::sleep};

#[main]
async fn main() {
    let mut player_manager = PlayerManager::new().await.unwrap();

    // if this doesnt work on here it wont work on js
    let mut rx = player_manager.set_events();
    loop {
        match rx.recv().await {
            Some(ManagerEvent::SessionsChanged) => {
                dbg!("SessionsChanged");
                player_manager.update_sessions(None);

                // just to test if it works
                sleep(Duration::from_millis(100)).await;
                if let Some(session) = player_manager.get_active_session() {
                    println!("{}", session.get_aumid());
                    session.pause().await;
                } else {
                    println!("No session 💀")
                }
            }
            Some(ManagerEvent::CurrentSessionChanged) => {
                dbg!("CurrentSessionChanged");
                player_manager.update_system_session();
            }
            None => (),
        }
    }
}
