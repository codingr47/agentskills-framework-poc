use std::error::Error;

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

pub fn run_chat_ui() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    ratatui::run(|terminal| -> Result<(), Box<dyn Error>> {
        loop {
            terminal.draw(|frame| {
                let [header_area, chat_area, input_area] = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(1),
                        Constraint::Length(3),
                    ])
                    .areas(frame.area());

                let header = Paragraph::new(Line::from("Welcome to the agentskills chat"))
                    .style(Style::default().fg(Color::Blue))
                    .block(Block::default().borders(Borders::ALL));

                let chat =
                    Paragraph::new("").block(Block::default().title("Chat").borders(Borders::ALL));

                let input_panel = Paragraph::new(input.as_str())
                    .block(Block::default().title("Message").borders(Borders::ALL));

                frame.render_widget(header, header_area);
                frame.render_widget(chat, chat_area);
                frame.render_widget(input_panel, input_area);
                frame
                    .set_cursor_position((input_area.x + input.len() as u16 + 1, input_area.y + 1));
            })?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match (key.code, key.modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => break Ok(()),
                    (KeyCode::Enter, _) => {}
                    (KeyCode::Backspace, _) => {
                        input.pop();
                    }
                    (KeyCode::Char(character), _) => {
                        input.push(character);
                    }
                    _ => {}
                }
            }
        }
    })
}
