use ratatui::{
    Terminal,
    backend::TestBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render_frame(
    width: u16,
    height: u16,
    nickname: &str,
    current_room: &str,
    channels: &[String],
    users: &[String],
    messages: &[String],
    input_buffer: &str,
) -> Vec<u8> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            // vertical split: main workspace + chatbox pinned to the bottom
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(f.size());

            // horizontal split: left sidebar + right chat log
            let main_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                .split(chunks[0]);

            // Sub-vertical split: sidebar split into channels and users
            let sidebar_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_layout[0]);

            // Render channels list
            let channel_items: Vec<ListItem> = channels
                .iter()
                .map(|c| ListItem::new(format!("# {}", c)))
                .collect();
            let channel_list = List::new(channel_items)
                .block(Block::default().borders(Borders::ALL).title(" Channels "));
            f.render_widget(channel_list, sidebar_chunks[0]);

            // Render users list
            let user_items: Vec<ListItem> = users
                .iter()
                .map(|u| ListItem::new(format!("@ {}", u)))
                .collect();
            let user_list = List::new(user_items)
                .block(Block::default().borders(Borders::ALL).title(" Users "));
            f.render_widget(user_list, sidebar_chunks[1]);

            // Render chat logs
            let chat_items: Vec<ListItem> =
                messages.iter().map(|m| ListItem::new(m.as_str())).collect();
            let chat_list = List::new(chat_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Chat: {} ", current_room)),
            );
            f.render_widget(chat_list, main_layout[1]);

            // Render bottom pinned chatbox
            let input_widget =
                Paragraph::new(format!("Message {} > {}", current_room, input_buffer)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" Active User: {} ", nickname)),
                );
            f.render_widget(input_widget, chunks[1]);
        })
        .unwrap();

    let mut output = b"\x1b[H\x1b[2J".to_vec();
    let buffer = terminal.backend().buffer();

    for y in 0..height {
        for x in 0..width {
            let cell = buffer.get(x, y);
            output.extend_from_slice(cell.symbol().as_bytes());
        }
        if y < height - 1 {
            output.extend_from_slice(b"\r\n");
        }
    }

    output
}
