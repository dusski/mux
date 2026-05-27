use crate::state::SharedState;
use russh::server::{Auth, Msg, Session};
use russh::{Channel, ChannelId};

#[derive(Clone)]
pub struct ChatSSHServer {
    pub state: SharedState,
}

impl russh::server::Server for ChatSSHServer {
    type Handler = ClientSession;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        ClientSession {
            state: self.state.clone(),
            nickname: String::from("Anonymous"),
            line_buffer: Vec::new(),
            current_room_sender: None,
            current_room_receiver: None,
        }
    }
}

pub struct ClientSession {
    pub state: SharedState,
    pub nickname: String,
    pub line_buffer: Vec<u8>,
    pub current_room_sender: Option<tokio::sync::broadcast::Sender<String>>,
    pub current_room_receiver: Option<tokio::sync::broadcast::Receiver<String>>,
}

impl russh::server::Handler for ClientSession {
    type Error = anyhow::Error;

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _public_key: &russh::keys::PublicKey,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        println!("A client successfully opened an SSH terminal channel.");

        let welcome_message = "Welcome to the Async SSH Chat Room!\r\nType something: ";

        _session.data(_channel.id(), welcome_message.as_bytes())?;
        Ok(true)
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        _data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // println!("Server received raw bytes from client: {:?}", _data);

        _session.data(_channel, _data.to_vec())?;

        for &byte in _data {
            if byte == b'\r' || byte == b'\n' {
                if self.line_buffer.is_empty() {
                    continue;
                }

                if let Ok(line_str) = std::str::from_utf8(&self.line_buffer) {
                    let trimmed = line_str.trim();

                    if trimmed.starts_with("/join") {
                        let room_name = trimmed.trim_start_matches("/join").trim();
                        if room_name.is_empty() {
                            _session.data(_channel, "Usage: /join <room_name>\r\n".as_bytes())?;
                            self.line_buffer.clear();
                            continue;
                        } else {
                            let sender = self.state.get_or_create_room(room_name).await;
                            let mut room_receiver = sender.subscribe();
                            self.current_room_sender = Some(sender);

                            let session_handle = _session.handle();

                            tokio::spawn(async move {
                                while let Ok(msg) = room_receiver.recv().await {
                                    if session_handle.data(_channel, msg.into_bytes()).await.is_err() {
                                        break;
                                    }
                                }
                            });

                            let confirm = format!("Joined room '{}'. You can start chatting!\r\n", room_name);
                            _session.data(_channel, confirm.into_bytes())?;
                            // self.current_room_receiver = Some(sender.subscribe());
                            // self.current_room_sender = Some(sender);
                            // let confirm =
                            //     format!("Joined room '{}'. You can start chatting!\r\n", room_name);
                            // _session.data(_channel, confirm.into_bytes())?;
                        }
                    } else if trimmed.starts_with("/nick") {
                        let parsed_nick = trimmed.trim_start_matches("/nick").trim();
                        if parsed_nick.is_empty() {
                            _session.data(_channel, "Usage: /nick <nickname>\r\n".as_bytes())?;
                            self.line_buffer.clear();
                        } else {
                            self.nickname = parsed_nick.to_string();
                            let confirm =
                                format!("Nickname set/changed to '{}'/\r\n", self.nickname);
                            _session.data(_channel, confirm.into_bytes())?;
                        }
                    } else if trimmed == "/rooms" {
                        let rooms = self.state.list_rooms().await;
                        let response = if rooms.is_empty() {
                            "No active rooms\r\n".to_string()
                        } else {
                            format!("Active rooms:\r\n{}\r\n", rooms.join("\r\n"))
                        };
                        _session.data(_channel, response.into_bytes())?;
                    } else if trimmed == "/quit" {
                        _session.data(_channel, "Goodbye!\r\n".as_bytes())?;
                        _session.close(_channel)?;
                    } else {
                        if let Some(sender) = &self.current_room_sender {
                            let msg = format!("[{}] {}\r\n", self.nickname, trimmed);
                            sender.send(msg)?;
                        } else {
                            _session.data(
                                _channel,
                                b"You must join a room first using /join <room_name>\r\n".to_vec(),
                            )?;
                        }
                    }
                }
                self.line_buffer.clear();
            } else if byte == 127 || byte == 0 {
                self.line_buffer.pop();
            } else {
                self.line_buffer.push(byte);
            }
        }

        Ok(())
    }
}
