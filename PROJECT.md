Building a TUI (Terminal User Interface) based Slack clone over SSH is an exceptional project for learning Rust. It forces you to dive into ownership, safe concurrency, async networking (`tokio`), and state management.

To achieve a true "Slack-like" experience with a persistent sidebar, split panes, and a dedicated bottom chatbox over a raw SSH stream, you will need to move away from manually printing raw strings like `> `. Instead, you need to issue ANSI terminal grid control sequences to draw a layout, or leverage a Rust layout engine like `ratatui` (the industry standard for Rust TUIs).

Here is the plain-text layout and project specifications file for your roadmap.

---

# Project Specification: SSH-Slack Terminal Clone

## Project Philosophy

The objective of this project is to build a terminal-centric communication platform mimicking core Slack mechanics, completely accessible via a standard native SSH client (`ssh localhost -p 2222`). This project serves as a comprehensive playground to master Rust systems programming, concurrent memory access, and custom terminal interface design.

## Core Visual Layout Mockup

The terminal window is split into separate logical panes using ANSI window-boxing routines. The dimensions adapt dynamically utilizing the client-side terminal resize event listener (`window_change`).

```text
+-----------------------+----------------------------------------------------+
|  # GENERAL            | [System]: Welcome to SSH-Slack!                    |
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
| #general > _                                                               |
+----------------------------------------------------------------------------+

```

## Functional Requirements

### 1. The Interactive Sidebar

* **Channel Directory**: Displays all active chat rooms prepended with a `#` prefix. Rooms are pulled dynamically from the synchronized room registry.
* **User Directory**: Displays all currently active SSH connections globally in the state registry, prepended with an `@` prefix.
* **Navigation Mechanics**: Users can toggle between typing a message and navigating the channel listing. Pressing a modifier (e.g., `Esc` or `Ctrl+O`) shifts cursor control to the sidebar, where `Up`/`Down` arrow keys let the user cycle rooms and hit `Enter` to switch contexts.

### 2. Pinned Bottom Chatbox

* **Dedicated Input Region**: The bottom-most lines of the terminal are reserved exclusively for string compilation. Standard chat messages do not push this box down; it stays pinned to the absolute bottom row (`terminal_height`).
* **Natural Typing Processing**: The key reader captures characters natively, supporting backspaces, left/right cursor navigation within the prompt, and standard carriage returns to emit messages.

### 3. Asynchronous Multi-Pane Canvas Re-Rendering

* **Local Input Echo**: Characters typed by the local client only redraw the bottom chatbox region, eliminating visual stuttering.
* **Background Broadcast Updates**: When a message arrives via a room's `broadcast::Receiver`, the background task recalculates the bounding box of the upper chat pane, clears only that sector using ANSI control vectors, appends the text, and restores the cursor cleanly back into the input field.

---

## Technical Architecture Stack & Milestones

### Milestone 1: State Upgrades (`state.rs`)

* Transition `SharedState` from a basic room mapper to a global matrix tracking:
* Active channel broadcast links.
* Registered active users, mapping their connection handles to their current location (e.g., tracking that user `Alice` is currently viewing `#rust-learning`).



### Milestone 2: TUI Layout Management (The "How to Draw" Decision)

To render the sidebar and chat boxes cleanly, the project must move in one of two directions:

* **Option A: Manual ANSI Grid Compositor**: Write a custom rendering engine using raw escape codes. For example, `\x1b[H` moves the cursor to the top-left, and `\x1b[Y;XH` moves the cursor to specific coordinates to draw partition dividers (`|`, `-`).
* **Option B: Ratatui Integration (Recommended)**: Use the `ratatui` crate to declare an application UI layout. It provides a `Layout` widget that breaks the terminal into constraints (e.g., `Constraint::Percentage(25)` for the sidebar, `Constraint::Percentage(75)` for the chat logs).

### Milestone 3: Frame Buffer Transmission

* Instead of directly piping small fragments of strings through `session_handle.data()`, the layout generator will compile an off-screen string representation of the entire terminal window frame whenever state shifts.
* The system performs a diff or pushes the frame string down the SSH pipeline in a single step to guarantee seamless animation updates across the client screen.