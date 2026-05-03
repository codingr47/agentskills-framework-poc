use std::{
    collections::HashMap,
    error::Error,
    io::stdout,
    sync::atomic::{AtomicU64, Ordering},
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use ratatui::{
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
            KeyModifiers, MouseEventKind,
        },
        execute,
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

pub type ChatResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

static NEXT_STREAM_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct ChatMessageHandler {
    sender: Sender<ChatMessageEvent>,
}

pub struct ChatMessageStream {
    id: u64,
    sender: Sender<ChatMessageEvent>,
}

pub struct ChatMessageReceiver {
    receiver: Receiver<ChatMessageEvent>,
}

enum ChatMessageEvent {
    Start { id: u64, content: String },
    Stream { id: u64, content: String },
    End { id: u64 },
}

pub fn chat_message_handler() -> (ChatMessageHandler, ChatMessageReceiver) {
    let (sender, receiver) = mpsc::channel();
    (
        ChatMessageHandler { sender },
        ChatMessageReceiver { receiver },
    )
}

impl ChatMessageHandler {
    pub fn user(&self) -> ChatMessageStream {
        self.start_stream("User: ".to_string())
    }

    pub fn assistant(&self) -> ChatMessageStream {
        self.start_stream("Assistant: ".to_string())
    }

    pub fn tool(
        &self,
        tool_name: impl Into<String>,
        arguments: serde_json::Value,
    ) -> ChatMessageStream {
        self.start_stream(format!(
            "Tool: A tool `{}` has been called with these arguments {}",
            tool_name.into(),
            arguments
        ))
    }

    fn start_stream(&self, content: String) -> ChatMessageStream {
        let id = NEXT_STREAM_ID.fetch_add(1, Ordering::Relaxed);
        let _ = self.sender.send(ChatMessageEvent::Start { id, content });

        ChatMessageStream {
            id,
            sender: self.sender.clone(),
        }
    }
}

impl ChatMessageStream {
    pub fn stream(&self, message: String) {
        let _ = self.sender.send(ChatMessageEvent::Stream {
            id: self.id,
            content: message,
        });
    }

    pub fn end(self) {
        let _ = self.sender.send(ChatMessageEvent::End { id: self.id });
    }
}

pub fn chat_message_channel() -> (ChatMessageHandler, ChatMessageReceiver) {
    chat_message_handler()
}

pub fn run_chat_ui<F>(incoming_messages: ChatMessageReceiver, mut on_enter: F) -> ChatResult<()>
where
    F: FnMut(String),
{
    let mut input = String::new();
    let mut chat_messages: Vec<String> = Vec::new();
    let mut active_streams: HashMap<u64, usize> = HashMap::new();
    let mut chat_scroll = 0;
    let mut follow_chat_output = true;
    let mut max_chat_scroll = 0;
    let mut visible_chat_rows = 1;

    execute!(stdout(), EnableMouseCapture)?;

    let result = ratatui::run(|terminal| -> ChatResult<()> {
        loop {
            while let Ok(event) = incoming_messages.receiver.try_recv() {
                match event {
                    ChatMessageEvent::Start { id, content } => {
                        chat_messages.push(content);
                        active_streams.insert(id, chat_messages.len() - 1);
                    }
                    ChatMessageEvent::Stream { id, content } => {
                        if let Some(message_index) = active_streams.get(&id) {
                            chat_messages[*message_index].push_str(&content);
                        }
                    }
                    ChatMessageEvent::End { id } => {
                        active_streams.remove(&id);
                    }
                }
            }

            terminal.draw(|frame| {
                let area = frame.area();
                let input_width = area.width.saturating_sub(2).max(1);
                let wrapped_input = wrap_input(&input, input_width);
                let input_rows = wrapped_input.lines().count().max(1) as u16;
                let max_input_height = area.height.saturating_sub(4).max(3);
                let input_height = (input_rows + 2).min(max_input_height);

                let [header_area, chat_area, input_area] = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(1),
                        Constraint::Length(input_height),
                    ])
                    .areas(area);

                let chat_width = chat_area.width.saturating_sub(2).max(1);
                visible_chat_rows = chat_area.height.saturating_sub(2).max(1);
                let chat_text = wrap_chat_messages(&chat_messages, chat_width);
                let chat_rows = chat_text.lines().count().max(1) as u16;
                max_chat_scroll = chat_rows.saturating_sub(visible_chat_rows);

                if follow_chat_output {
                    chat_scroll = max_chat_scroll;
                } else {
                    chat_scroll = chat_scroll.min(max_chat_scroll);
                }

                let header = Paragraph::new(Line::from("Welcome to the agentskills chat"))
                    .style(Style::default().fg(Color::Blue))
                    .block(Block::default().borders(Borders::ALL));

                let chat = Paragraph::new(chat_text)
                    .block(Block::default().title("Chat").borders(Borders::ALL))
                    .scroll((chat_scroll, 0));

                let visible_input_rows = input_area.height.saturating_sub(2).max(1);
                let cursor_row = input.chars().count() as u16 / input_width;
                let cursor_column = input.chars().count() as u16 % input_width;
                let input_scroll = input_rows.saturating_sub(visible_input_rows);
                let cursor_visible_row = cursor_row.saturating_sub(input_scroll);

                let input_panel = Paragraph::new(wrapped_input)
                    .block(Block::default().title("Message").borders(Borders::ALL))
                    .scroll((input_scroll, 0));

                frame.render_widget(header, header_area);
                frame.render_widget(chat, chat_area);
                frame.render_widget(input_panel, input_area);
                frame.set_cursor_position((
                    input_area.x + cursor_column + 1,
                    input_area.y + cursor_visible_row + 1,
                ));
            })?;

            if !event::poll(Duration::from_millis(50))? {
                continue;
            }

            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match (key.code, key.modifiers) {
                        (KeyCode::Char('x'), KeyModifiers::CONTROL) => break Ok(()),
                        (KeyCode::PageUp, _) => {
                            chat_scroll = chat_scroll.saturating_sub(visible_chat_rows);
                            follow_chat_output = false;
                        }
                        (KeyCode::PageDown, _) => {
                            chat_scroll = chat_scroll
                                .saturating_add(visible_chat_rows)
                                .min(max_chat_scroll);
                            follow_chat_output = chat_scroll == max_chat_scroll;
                        }
                        (KeyCode::End, _) => {
                            follow_chat_output = true;
                        }
                        (KeyCode::Enter, _) => {
                            let submitted_message = std::mem::take(&mut input);
                            on_enter(submitted_message);
                            follow_chat_output = true;
                        }
                        (KeyCode::Backspace, _) => {
                            input.pop();
                        }
                        (KeyCode::Char(character), _) => {
                            input.push(character);
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        chat_scroll = chat_scroll.saturating_sub(3);
                        follow_chat_output = false;
                    }
                    MouseEventKind::ScrollDown => {
                        chat_scroll = chat_scroll.saturating_add(3).min(max_chat_scroll);
                        follow_chat_output = chat_scroll == max_chat_scroll;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    });

    execute!(stdout(), DisableMouseCapture)?;
    result
}

fn wrap_input(input: &str, width: u16) -> String {
    let width = width.max(1) as usize;
    let mut wrapped = String::new();

    for (index, character) in input.chars().enumerate() {
        if index > 0 && index % width == 0 {
            wrapped.push('\n');
        }

        wrapped.push(character);
    }

    wrapped
}

fn wrap_chat_messages(messages: &[String], width: u16) -> String {
    messages
        .iter()
        .map(|message| wrap_text(message, width))
        .collect::<Vec<_>>()
        .join("\n")
}

fn wrap_text(text: &str, width: u16) -> String {
    let width = width.max(1) as usize;
    let mut wrapped = String::new();
    let mut column = 0;

    for character in text.chars() {
        if character == '\n' {
            wrapped.push(character);
            column = 0;
            continue;
        }

        if column >= width {
            wrapped.push('\n');
            column = 0;
        }

        wrapped.push(character);
        column += 1;
    }

    wrapped
}
