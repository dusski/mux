# Interactive SSH Chat Server Enhancement Plan

This document tracks the implemented architectural updates transforming the `russh`-based server from a raw byte-echo server into an asynchronous, multi-pane workspace driven by Ratatui over a secure SSH pipeline.

## Summary of Completed Architecture Updates

1. **Persistent Host Key Correction (`main.rs`)**: Fixed runtime startup crashes caused by syntax mismatches when encoding private keys in PKCS8 format. Updated generation logic to output standard, explicitly formatted OpenSSH key strings via `.to_openssh(LineEnding::LF)` for seamless reboots.
2. **Ratatui Presentation Layer (`renderer.rs`)**: Abstracted all UI drawing logic away from the networking code. Created a pure, deterministic `render_frame` module that uses a virtual `TestBackend` to split the terminal into a Slack-like grid: a left-hand Sidebar (Channels and Users), a right-hand Main Chat log, and a fixed-height Pinned Chatbox at the bottom.
3. **Atomic Frame Buffer Serialization (`renderer.rs`)**: Overcame partial line-clearing bugs by converting Ratatui's virtual grid coordinates to an immutable vector layout. The module flushes a screen-wipe command (`\x1b[H\x1b[2J`) followed by the complete screen frame as a single, atomic byte array to ensure stutter-free animation updates.
4. **Isolated Canvas State Tracking (`ssh_server.rs`)**: Extended the `ClientSession` struct to maintain separate connection states, including dedicated line-buffers, nicknames, and real-time screen boundaries (`terminal_width` / `terminal_height`).

---

## Direct Code Insertions & Event Loop Alignment

### 1. Unified Client Session Properties

The connection handler isolates layout metrics independently for every concurrent SSH user session, preventing visual regressions across shared display contexts.

```rust
pub struct ClientSession {
    pub state: SharedState,
    pub nickname: String,
    pub current_room: String,
    pub line_buffer: Vec<u8>,
    pub terminal_width: u32,
    pub terminal_height: u32,
}
```

### 2. Streamlined Rendering Method Hook
Rather than executing raw text echoes, the server invokes an encapsulated `render_and_flush` block inside `ssh_server.rs` to process data from the state registries and synchronize the remote client view.

```rust
impl ClientSession {
    async fn render_and_flush(
        &self,
        _session: &mut russh::server::Session,
        _channel: russh::ChannelId
    ) -> Result<(), russh::Error> {
        let channels = self.state.list_rooms().await;
        let users = self.state.list_users().await;
        let messages = self.state.get_room_history(&self.current_room).await;
        let current_input = std::str::from_utf8(&self.line_buffer).unwrap_or("");

        let frame_bytes = crate::renderer::render_frame(
            self.terminal_width as u16,
            self.terminal_height as u16,
            &self.nickname,
            &self.current_room,
            &channels,
            &users,
            &messages,
            current_input,
        );

        _session.data(_channel, frame_bytes.into())?;
        Ok(())
    }
}
```

### 3. Keystroke Stream Processing
The incoming data parser maps user keys directly to mutations within the local vector buffer before triggering an off-screen layout repaint.

```rust
for &byte in _data {
    if byte == b'\r' || byte == b'\n' {
        // TODO: Wire up command parsing (/join, /nick) and message distribution loops
        self.line_buffer.clear();
        let _ = self.render_and_flush(_session, _channel).await;
    } else if byte == 127 || byte == 8 {
        // --- ATOMIC BACKSPACE HANDLING ---
        if !self.line_buffer.is_empty() {
            self.line_buffer.pop();
            let _ = self.render_and_flush(_session, _channel).await;
        }
    } else {
        // --- NORMAL CHARACTER FILL ---
        if byte >= 32 && byte <= 126 {
            self.line_buffer.push(byte);
            let _ = self.render_and_flush(_session, _channel).await;
        }
    }
}
```

### 4. Dynamic Window Resizing Callback
When physical browser or terminal frames stretch, the server captures the event protocol packet, registers the columns/rows, and commands an instant UI redraw.

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
    self.terminal_width = col;
    self.terminal_height = row;

    let _ = self.render_and_flush(_session, _channel).await;
    Ok(())
}
```

---

## Remaining Development Milestones

- [ ] **Milestone 1 State Upgrades**: Expand `SharedState` with safe async concurrency locks (`RwLock`) to keep global track of who is active and what specific channels they are looking at.
- [ ] **Command Parsing Engine**: Restore command interceptors inside the Enter key handler block to parse incoming strings for `/join <room>`, `/nick <name>`, and `/rooms`.
- [ ] **Background Broadcast Redraws**: Wire up an async background subscriber thread to listen for room broadcasts and trigger a frame re-render for clients whenever an external message arrives.