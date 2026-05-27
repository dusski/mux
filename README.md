# Mux

A terminal-centric, Slack-like workspace collaboration platform delivered securely over an SSH pipeline and written entirely in Rust.

Project Mux transforms the classic, retro command-line experience into a modern team communication hub. By multiplexing individual client streams through a secure SSH server into asynchronous Tokio broadcast channels, it provides a persistent multi-pane workspace directly inside any native terminal emulator.

---

## Core Visual Layout

The terminal window splits into separate logical panes using ANSI window-boxing routines. The coordinates and boundary dimensions adapt dynamically per client by intercepting the client-side SSH `window_change` protocol event.

```text
+-----------------------+----------------------------------------------------+
|  # general            | [System]: Welcome to Project Mux!                  |
|  # rust-learning      | [Alice]: Has anyone checked out tokio channels?    |
|  # development        | [Bob]: Yeah, broadcast loops work perfectly here.  |
|                       | [Alice]: Dynamic layout resizing is next!          |
|                       |                                                    |
|                       |                                                    |
|                       |                                                    |
|                       |                                                    |
|-----------------------|                                                    |
|  @ Alice              |                                                    |
|  @ Bob                |                                                    |
|  @ Anonymous (You)    |                                                    |
+-----------------------+----------------------------------------------------+
| Message #general > _                                                       |
+----------------------------------------------------------------------------+

```

---

## Architecture Features

* **Persistent Host Keys**: Utilizes an explicitly formatted OpenSSH key string layout (`to_openssh`) preventing host identification mismatch errors upon server restarts.
* **Per-Client State Management**: Tracks isolated nicknames, line-buffers, and unique terminal dimensions (`terminal_width` / `terminal_height`) independently for every concurrent SSH connection.
* **Visual Erasure Engine**: Implements a three-byte ASCII escape sequence loop (`\x08 \x08`) providing proper backspace text elimination over raw bytes.
* **Asynchronous Multi-Pane Re-Rendering**: Employs an ANSI line-clearing control vector (`\x1b[2K\r`) within a background `tokio::spawn` loop to seamlessly print incoming room broadcasts without disrupting a user's active prompt input.

---

## Development Roadmap

* **Milestone 1**: Transition `SharedState` to track active global channels and map registered user locations.
* **Milestone 2**: Interface infrastructure expansion using a manual ANSI compositor layout or native `ratatui` UI layout widgets.
* **Milestone 3**: Off-screen frame buffer compilation to push entire terminal window frames down the SSH pipeline in an atomized single step.

---

## Quick Start

### 1. Wipe old artifacts and launch the server

```bash
rm -f host_key.pem
cargo run

```

### 2. Reset client-side cached entries

```bash
ssh-keygen -f "~/.ssh/known_hosts" -R "[127.0.0.1]:2222"

```

### 3. Connect via any standard terminal

```bash
ssh 127.0.0.1 -p 2222

```