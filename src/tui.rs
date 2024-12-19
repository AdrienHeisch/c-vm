use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::Stylize,
    widgets::Paragraph,
    DefaultTerminal,
};

pub fn start() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    loop {
        draw(&mut terminal)?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn draw(terminal: &mut ratatui::Terminal<ratatui::prelude::CrosstermBackend<io::Stdout>>) -> Result<(), io::Error> {
    terminal.draw(|frame| {
        let greeting = Paragraph::new("Hello Ratatui! (press 'q' to quit)")
            .white();
        frame.render_widget(greeting, frame.area());
    })?;
    
    Ok(())
}