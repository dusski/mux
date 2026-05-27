// use crate::state::SharedState;
// use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
// use tokio::net::TcpStream;
// use tokio::sync::{broadcast, watch};

// pub async fn handle_connection(
//     socket: TcpStream,
//     state: SharedState,
//     mut shutdown_receiver: watch::Receiver<bool>,
// ) {
//     let (reader, mut writer) = socket.into_split();
//     let mut reader = BufReader::new(reader);
//     let mut line = String::new();

//     let mut nickname = String::from("Anonymous");

//     let mut current_room_sender: Option<broadcast::Sender<String>> = None;
//     let mut current_room_receiver: Option<broadcast::Receiver<String>> = None;

//     loop {
//         tokio::select! {
//             bytes_read = reader.read_line(&mut line) => {
//               match bytes_read {
//                 Ok(0) | Err(_) => break,
//                 Ok(_) => {
//                   let trimmed = line.trim();

//                   if trimmed.starts_with("/join") {
//                     let room_name = trimmed.trim_start_matches("/join").trim();
//                     if room_name.is_empty() {
//                       let _ = writer.write_all(b"Usage: /join <room_name>\n").await;
//                       line.clear();
//                       continue;
//                     }
//                     let sender = state.get_or_create_room(room_name).await;
//                     let receiver = sender.subscribe();
//                     current_room_sender = Some(sender.clone());
//                     current_room_receiver = Some(receiver);
//                   }
//                   else if trimmed.starts_with("/nick") {
//                     let parsed_nick = trimmed.trim_start_matches("/nick").trim();
//                     if parsed_nick.is_empty() {
//                       let _ = writer.write_all(b"Usage: /nick <nickname>\n").await;
//                       line.clear();
//                       continue;
//                     }
//                     nickname = parsed_nick.to_string();
//                   } else if trimmed == "/rooms" {
//                     let rooms = state.list_rooms().await;
//                     let response = if rooms.is_empty() {
//                       "No active rooms\n".to_string()
//                     } else {
//                       format!("Active rooms:\n{}\n", rooms.join("\n"))
//                     };
//                     let _ = writer.write_all(response.as_bytes()).await;
//                   } else if trimmed == "/quit" {
//                     break;
//                   }
//                   else {
//                     if let Some(sender) = &current_room_sender {
//                       let msg = format!("[{}] {}\n", nickname, trimmed);
//                       let _ = sender.send(msg);
//                     } else {
//                       let _ = writer.write_all(b"You must join a room first using /join <room name>\n").await;
//                     }
//                   }

//                   line.clear()
//                 }
//               }
//             }

//             Ok(msg) = async {
//               current_room_receiver.as_mut().unwrap().recv().await
//             }, if current_room_receiver.is_some() => {
//               let _ = writer.write_all(msg.as_bytes()).await;
//             }

//             _ = shutdown_receiver.changed() => {
//               break;
//             }
//         }
//     }
// }
