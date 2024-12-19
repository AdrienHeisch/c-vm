use crate::{instruction::Instruction, vm::VM};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Terminal,
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
        draw(&mut terminal, program, vm.pc() as usize, &vm.show_ram())?;

        while let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => {
                        if let Some(instruction) = program.get(vm.pc() as usize) {
                            if let Some(exit_code) = vm.execute(*instruction) {
                                println!("Program exited with code : {exit_code}");
                                return Ok(());
                            }
                        } else {
                            return Ok(());
                        }
                    },
                    _ => continue,
                }
                break;
            }
        }
    }
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    program: &[Instruction],
    pc: usize,
    ram: &[String],
) -> Result<(), io::Error> {

    terminal.draw(|frame| {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(64)])
            .split(frame.area());

        let program_display = Paragraph::new(format_program(program, pc))
            .white()
            .block(Block::new().borders(Borders::ALL));
        frame.render_widget(program_display, layout[0]);

        let ram_display = Paragraph::new(format_ram(ram))
            .white()
            .block(Block::new().borders(Borders::ALL));
        frame.render_widget(ram_display, layout[1]);
    })?;

    Ok(())
}

fn format_program(program: &[Instruction], pc: usize) -> String {
    let mut string = String::new();
    for (idx, str) in program.iter().map(|i| format!("{i:?}")).enumerate() {
        if idx == pc {
            string.push_str(" > ");
        } else {
            string.push_str("   ");
        }
        string.push_str(&str);
        string.push('\n');
    }
    string
}

fn format_ram(ram: &[String]) -> String {
    let mut string = String::new();
    string.push_str("            ");
    string.push_str(
        &(0..16)
            .map(|i| format!("{:02X}", i))
            .collect::<Vec<_>>()
            .join(" "),
    );
    for (idx, str) in ram.iter().enumerate() {
        if idx % 16 == 0 {
            string.push_str(&format!("\n {:08X}   ", idx));
        } else {
            string.push(' ');
        }
        string.push_str(str);
    }
    string
}
