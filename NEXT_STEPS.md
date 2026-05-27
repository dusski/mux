# Interactive SSH Chat Server Enhancement Plan

This document outlines the implemented updates to transform the `russh`-based server from a raw byte-echo server into an interactive, stateful chat terminal featuring proper backspace erasure, real-time command parsing, username prepending, and a dynamic buffer prompt.

## Summary of Completed Architecture Updates

1. **Persistent Host Key Correction (`main.rs`)**: Fixed the runtime startup crash (`os error 2 / NotFound`) caused by syntax mismatches when encoding raw Private Keys with PKCS8 format. Updated the generation logic to output standard, explicitly formatted OpenSSH key string wrappers via `.to_openssh(LineEnding::LF)` which are 100% compatible with `russh::keys::load_secret_key` upon subsequent reboots.
2. **Interactive Input Prompts (`ssh_server.rs`)**: Appended an explicit, responsive `> ` visual text decorator to the connection workflow, rendering immediately upon a successful handshake and automatically rebuilding itself after every command evaluation or message submission.
3. **Visual Backspace and Control-Sieve Engine (`ssh_server.rs`)**: Solved the "ghost delete" issue where backspace keystrokes deleted characters in the internal memory buffer but left them as visual remnants on the user's terminal window. Wired a three-byte ASCII escape translation block (`\x08 \x08` - Back, Space, Back) to erase characters cleanly on the terminal, alongside a bounds-check filtering out non-printable ASCII control bytes.
4. **Dynamic Asynchronous Broadcast Recipient Task (`ssh_server.rs`)**: Integrated a background `tokio::spawn` worker routine attached to the `/join` execution branch. This task utilizes an abstracted `_session.handle()` clone to listen non-blockingly to `tokio::sync::broadcast` messages from the room, employing an ANSI line-clearing escape sequence (`\x1b[2K\r`) to cleanly wipe out a user's active prompt line, echo incoming chat room broadcasts, and redraw the active input decorator instantly beneath it.
5. **Per-Client Dynamic Terminal Resizing Architecture**: Integrated support for intercepting real-time SSH `window_change` protocol packets. This layout isolates tracking properties down to individual, per-connection `ClientSession` state boundaries, allowing users on diverse physical displays to update personal canvas parameters without triggering visual regressions across shared active room channels.

## Direct Code Insertions and Deletions

### 1. `main.rs` File Modifications

#### **DELETION**

```rust
let new_key = russh::keys::PrivateKey::random(
    &mut russh::keys::key::safe_rng(),
    russh::keys::Algorithm::Ed25519,
)?;
let mut pem_string = Vec::new();
russh::keys::encode_pkcs8_pem(&new_key, &mut pem_string)?;
std::fs::write(key_path, pem_string)?;
println!("Generated new host key and saved to {:?}", key_path);
new_key

```

#### **INSERTION**

```rust
let new_key = russh::keys::PrivateKey::random(
    &mut russh::keys::key::safe_rng(),
    russh::keys::Algorithm::Ed25519,
)?;
// Convert using the proper ssh-key crate function visible in your menu
let openssh_key_string = new_key.to_openssh(russh::keys::ssh_key::LineEnding::LF)?;
std::fs::write(key_path, openssh_key_string.as_bytes())?;
println!("Generated new host key and saved to {:?}", key_path);
new_key

```

---

### 2. `ssh_server.rs` File Modifications

#### Welcome Message Adjustment

##### **DELETION**

```rust
let welcome_message = "Welcome to the Async SSH Chat Room!\r\nType something: ";

```

##### **INSERTION**

```rust
let welcome_message = "Welcome to the Async SSH Chat Room!\r\n> ";

```

#### Struct Definition and Connection Initializer Update

##### **DELETION**

```rust
pub struct ClientSession {
    pub state: SharedState,
}

// Inside impl russh::server::Server for ChatSSHServer:
fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
    ClientSession {
        state: self.state.clone(),
    }
}

```

##### **INSERTION**

```rust
pub struct ClientSession {
    pub state: SharedState,
    pub nickname: String,
    pub line_buffer: Vec<u8>,
    pub current_room_sender: Option<tokio::sync::broadcast::Sender<String>>,
    pub current_room_receiver: Option<tokio::sync::broadcast::Receiver<String>>,
    pub terminal_width: u32,
    pub terminal_height: u32,
}

// Inside impl russh::server::Server for ChatSSHServer:
fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
    ClientSession {
        state: self.state.clone(),
        nickname: String::from("Anonymous"),
        line_buffer: Vec::new(),
        current_room_sender: None,
        current_room_receiver: None,
        terminal_width: 80,
        terminal_height: 24,
    }
}

```

#### Main Processing and Event Loop Remap

##### **DELETION**

```rust
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
                    self.current_room_receiver = Some(sender.subscribe());
                    self.current_room_sender = Some(sender);
                    let confirm =
                        format!("Joined room '{}'. You can start chatting!\r\n", room_name);
                    _session.data(_channel, confirm.into_bytes())?;
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

```

##### **INSERTION**

```rust
for &byte in _data {
    if byte == b'\r' || byte == b'\n' {
        if self.line_buffer.is_empty() {
            // If they just hit enter on an empty line, give them a fresh prompt
            _session.data(_channel, b"\r\n> ")?;
            continue;
        }

        if let Ok(line_str) = std::str::from_utf8(&self.line_buffer) {
            let trimmed = line_str.trim();

            if trimmed.starts_with("/join") {
                let room_name = trimmed.trim_start_matches("/join").trim();
                if room_name.is_empty() {
                    _session.data(_channel, b"\r\nUsage: /join <room_name>\r\n> ")?;
                } else {
                    let sender = self.state.get_or_create_room(room_name).await;
                    let mut room_receiver = sender.subscribe();
                    self.current_room_sender = Some(sender);

                    let session_handle = _session.handle();
                    tokio::spawn(async move {
                        while let Ok(msg) = room_receiver.recv().await {
                            // Erase current prompt line, print message, re-add prompt
                            // \x1b[2K clears the current line, \r returns cursor to start
                            let broadcast_delivery = format!("\x1b[2K\r{}{}", msg, "> ");
                            if session_handle.data(_channel, broadcast_delivery.into_bytes()).await.is_err() {
                                break;
                            }
                        }
                    });

                    let confirm = format!("\r\nJoined room '{}'. You can start chatting!\r\n", room_name);
                    _session.data(_channel, confirm.into_bytes())?;
                }
            } else if trimmed.starts_with("/nick") {
                let parsed_nick = trimmed.trim_start_matches("/nick").trim();
                if parsed_nick.is_empty() {
                    _session.data(_channel, b"\r\nUsage: /nick <nickname>\r\n> ")?;
                } else {
                    self.nickname = parsed_nick.to_string();
                    let confirm = format!("\r\nNickname set to '{}'\r\n", self.nickname);
                    _session.data(_channel, confirm.into_bytes())?;
                }
            } else if trimmed == "/rooms" {
                let rooms = self.state.list_rooms().await;
                let response = if rooms.is_empty() {
                    "\r\nNo active rooms\r\n".to_string()
                } else {
                    format!("\r\nActive rooms:\r\n{}\r\n", rooms.join("\r\n"))
                };
                _session.data(_channel, response.into_bytes())?;
            } else if trimmed == "/quit" {
                _session.data(_channel, b"\r\nGoodbye!\r\n")?;
                _session.close(_channel)?;
                return Ok(());
            } else {
                if let Some(sender) = &self.current_room_sender {
                    let msg = format!("[{}]: {}\r\n", self.nickname, trimmed);
                    let _ = sender.send(msg);
                } else {
                    _session.data(_channel, b"\r\nYou must join a room first using /join <room_name>\r\n")?;
                }
            }
        }
        self.line_buffer.clear();
        // Always append a fresh input decoration prompt after executing a command/message
        _session.data(_channel, b"> ")?;

    } else if byte == 127 || byte == 8 {
        // --- BACKSPACE HANDLING ---
        if !self.line_buffer.is_empty() {
            self.line_buffer.pop();
            // Send the terminal sequence: Move Back, Write Space (Erase), Move Back
            _session.data(_channel, b"\x08 \x08")?;
        }
    } else {
        // Only echo characters and save them if they aren't control bytes
        if byte >= 32 && byte <= 126 {
            self.line_buffer.push(byte);
            _session.data(_channel, &[byte])?;
        }
    }
}

```

#### Window Size Trait Implementation Addition

##### **INSERTION (New Trait Callback Branch)**

```rust
  async fn window_change(
    &mut self,
    _channel: ChannelId,
    col: u32,
    row: u32,
    _width: u32,
    _height: u32,
    _session: &mut Session,
  ) -> Result<(), Self::Error> {
    // Dynamically store isolated client grid properties to drive future line wrapping engines
    self.terminal_width = col;
    self.terminal_height = row;
    Ok(())
  }

```

---

## Key Takeaway Checklist Before Booting

* [ ] Run `rm host_key.pem` inside your root project folder. This forces your server to rebuild a key using the fixed `to_openssh` format on initialization.
* [ ] Reset your client's stale memory entry via `ssh-keygen -f "~/.ssh/known_hosts" -R "[127.0.0.1]:2222"`.
* [ ] Run `cargo run`, open up multiple separate shells, and execute `/join internal-test` to experience full, visual room chatting!