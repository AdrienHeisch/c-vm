use crate::{instruction::Instruction, vm::{RAM_LEN, VM}, REG_LEN};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Terminal,
};
use std::{io, ops::Range};

pub fn run(program: &[Instruction]) -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = start(terminal, program);
    ratatui::restore();
    app_result
}

fn start(mut terminal: DefaultTerminal, program: &[Instruction]) -> io::Result<()> {
    let mut vm = VM::new();
    let mut next_instruction = None;
    let mut display_pc = 0;
    loop {
        draw(&mut terminal, program, &vm, display_pc)?;

        while let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => {
                        if let Some(instruction) = next_instruction {
                            if let Some(exit_code) = vm.execute(instruction) {
                                println!("Program exited with code : {exit_code}");
                                return Ok(());
                            }
                            next_instruction = None;
                        } else if let Some(instruction) = program.get(vm.pc() as usize) {
                            next_instruction = Some(*instruction);
                            display_pc = vm.pc() as usize;
                        } else {
                            return Ok(());
                        }
                    }
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
    vm: &VM,
    pc: usize,
) -> Result<(), io::Error> {
    terminal.draw(|frame| {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Length(28),
                Constraint::Length(62),
                Constraint::Fill(0),
            ])
            .split(frame.area());

        let program_str = program.iter().map(|i| format!("{i:?}")).collect::<Vec<_>>();
        let program_display = Paragraph::new(format_program(&program_str, pc))
            .white()
            .block(Block::new().title("Program").borders(Borders::ALL));
        frame.render_widget(program_display, layout[1]);

        let mem_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(6), Constraint::Fill(1)])
            .split(layout[2]);

        let regs = vm.show_regs();
        let target_regs = program.get(pc).unwrap().target_regs();
        let regs_display = Paragraph::new(format_regs(&regs, target_regs))
            .white()
            .block(Block::new().title("Registers").borders(Borders::ALL));
        frame.render_widget(regs_display, mem_layout[0]);

        let ram_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(50)])
            .split(mem_layout[1]);

        let ram_address = Paragraph::new(addresses(0..RAM_LEN / 16)).white().block(
            Block::new()
                .title("RAM")
                .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM),
        );
        frame.render_widget(ram_address, ram_layout[0]);

        let ram = vm.show_ram();
        let target_ram = program.get(pc).unwrap().target_ram();
        let ram_display = Paragraph::new(format_ram(vm, &ram, target_ram))
            .white()
            .block(Block::new().borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM));
        frame.render_widget(ram_display, ram_layout[1]);
    })?;

    Ok(())
}

fn format_program(program: &[String], pc: usize) -> Text {
    let mut lines = Vec::new();
    let mut spans = Vec::new();
    for (idx, str) in program.iter().enumerate() {
        if idx == pc {
            spans.push(Span::raw(" > "));
            spans.push(Span::styled(str, Style::default().black().on_white()));
        } else {
            spans.push(Span::raw("   "));
            spans.push(Span::raw(str));
        }
        lines.push(Line::from(spans));
        spans = Vec::new();
    }
    Text::from(lines)
}

fn format_regs(regs: &[String], (dst, src): (Vec<usize>, Vec<usize>)) -> Text {
    let mut lines = Vec::new();

    let primary_st = Style::default().black().on_white();
    let secondary_st = Style::default().black().on_dark_gray();
    let mut spans = Vec::new();
    for (idx, str) in regs.iter().enumerate() {
        let pos = if idx > 6 { idx + 1 } else { idx };

        if pos % 4 == 0 {
            spans.push(Span::raw(" "));
        } else {
            spans.push(Span::raw("    "));
        }

        spans.push(if dst.contains(&idx) {
            Span::styled(str, primary_st)
        } else if src.contains(&idx) {
            Span::styled(str, secondary_st)
        } else {
            Span::raw(str)
        });

        if pos % 4 == 3 {
            lines.push(Line::from(spans));
            spans = Vec::new();
        }

        if pos == 6 {
            lines.push(Line::from(spans));
            spans = Vec::new();
        }
    }

    lines.push(Line::from(spans));

    Text::from(lines)
}

// FIXME on stack operations, SP should visually stay at the original location
fn format_ram<'a>(vm: &VM, ram: &'a [String], targets: Vec<(bool, u64, bool)>) -> Text<'a> {
    let mut lines = Vec::new();

    lines.push(Line::raw(
        (0..16)
            .map(|i| format!("{:02X}", i))
            .collect::<Vec<_>>()
            .join(" "),
    ));

    let write_style = Style::default().black().on_white();
    let read_style = Style::default().black().on_dark_gray();
    let mut keep_highlighting = 0;
    let mut spans = Vec::new();
    let mut current_style = Style::default();
    for (idx, str) in ram.iter().enumerate() {
        let mut style = if keep_highlighting > 0 {
            keep_highlighting -= 1;
            current_style
        } else {
            Style::default()
        };

        if idx % 16 != 0 {
            spans.push(Span::styled(" ", style));
        }

        if let Some((_, _, write)) = targets
            .iter()
            .find(|(rfl, val, _)| idx == if *rfl { vm.get_reg(*val) } else { *val } as usize)
        {
            if *write {
                current_style = write_style;
            } else {
                current_style = read_style;
            }
            style = current_style;
            keep_highlighting = REG_LEN - 1;
        }

        spans.push(Span::styled(str, style));

        if idx % 16 == 15 {
            lines.push(Line::from(spans));
            spans = Vec::new();
        }
    }

    Text::from(lines)
}

fn addresses(range: Range<usize>) -> String {
    let mut string = String::new();
    string.push_str("            ");
    string.push_str("            ");
    for i in range {
        string.push_str(&format!("\n {:08X}   ", i));
    }
    string
}
