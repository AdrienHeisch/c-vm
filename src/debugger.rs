use crate::{instruction::Instruction, vm::VM};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::Stylize,
    widgets::Paragraph,
    DefaultTerminal,
};
use std::io;

pub fn run(program: &[Instruction]) -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = start(terminal, program);
    ratatui::restore();
    app_result
}

fn start(mut terminal: DefaultTerminal, program: &[Instruction]) -> io::Result<()> {
    let mut vm = VM::new();
    loop {
        draw(&mut terminal)?;

        while let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => (),
                    _ => continue,
                }
                break;
            }
        }
    }
}

fn draw(
    terminal: &mut ratatui::Terminal<ratatui::prelude::CrosstermBackend<io::Stdout>>,
) -> Result<(), io::Error> {
    terminal.draw(|frame| {
        let greeting = Paragraph::new("Hello Ratatui! (press space to quit)").white();
        frame.render_widget(greeting, frame.area());
    })?;

    Ok(())
}
