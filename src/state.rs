use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

#[derive(Clone)]
pub struct SharedState {
  rooms: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

impl SharedState {
  pub fn new() -> Self {
    Self {
      rooms: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub async fn get_or_create_room(&self, room_name: &str) -> broadcast::Sender<String> {
    let mut rooms_lock = self.rooms.lock().await;

    match rooms_lock.get(room_name) {
      Some(sender) => sender.clone(),
      None => {
        let (sender, _) = broadcast::channel(32);
        rooms_lock.insert(room_name.to_string(), sender.clone());
        sender
      }
    }
  }

  pub async fn list_rooms(&self) -> Vec<String> {
    let rooms_lock = self.rooms.lock().await;
    let room_names = rooms_lock.keys();
    room_names.cloned().collect()
  }
}