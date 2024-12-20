use crate::{instruction::Instruction, uvm, vm::VM, REG_LEN};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Terminal,
};
use std::{io, time::Duration};

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
    let mut auto = false;
    let mut done = false;
    let mut history = Vec::new();

    if let Some(instruction) = program.get(vm.pc() as usize) {
        next_instruction = Some(*instruction);
        display_pc = vm.pc() as usize;
    }

    loop {
        draw(
            &mut terminal,
            next_instruction.is_some(),
            &vm,
            program,
            display_pc,
            &history,
        )?;

        if event::poll(Duration::from_millis(33))? {
            while let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('r') => {
                            done = false;
                            vm = VM::new();
                            history = Vec::new();
                            if let Some(instruction) = program.get(vm.pc() as usize) {
                                next_instruction = Some(*instruction);
                                display_pc = vm.pc() as usize;
                            }
                        }
                        KeyCode::Enter => auto = !auto,
                        KeyCode::Char(' ') => {
                            auto = false;
                            if done {
                                continue;
                            }
                            if let Some(instruction) = next_instruction {
                                if let Some(exit_code) = vm.execute(instruction) {
                                    history.push(Line::raw(format!(
                                        "Program exited with code : {exit_code}"
                                    )));
                                    done = true;
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
                    if key.code != KeyCode::Enter {
                        auto = false;
                    }
                    break;
                }
            }
        }

        if auto && !done {
            if let Some(instruction) = program.get(vm.pc() as usize) {
                if let Some(exit_code) = vm.execute(*instruction) {
                    history.push(Line::raw(format!("Program exited with code : {exit_code}")));
                    done = true;
                }
                next_instruction = Some(*instruction);
                display_pc = vm.pc() as usize;
            }
        }

        vm.stdout()
            .lines()
            .for_each(|l| history.push(Line::raw(l.to_owned())));

        vm.stderr()
            .lines()
            .for_each(|l| history.push(Line::raw(format!("!!!>   {}", l))));
    }
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mode: bool,
    vm: &VM,
    program: &[Instruction],
    pc: usize,
    history: &[Line],
) -> Result<(), io::Error> {
    terminal.draw(|frame| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
            .split(frame.area());

        let controls = Paragraph::new(" Quit [q]   Reset [r]   Step [SPACE]   Run/Stop [ENTER]");
        frame.render_widget(controls, layout[1]);

        let hlayout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(0),
                Constraint::Length(32),
                Constraint::Length(62),
                Constraint::Fill(1),
                Constraint::Fill(0),
            ])
            .split(layout[0]);

        let program_str = program.iter().map(|i| format!("{i:?}")).collect::<Vec<_>>();
        let program_jmp = program.get(pc).unwrap().target_addr();
        let program_display =
            Paragraph::new(format_program(vm, &program_str, mode, pc, program_jmp))
                // .white()
                .block(Block::new().title("Program").borders(Borders::ALL));
        frame.render_widget(program_display, hlayout[1]);

        let mem_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(6), Constraint::Fill(1)])
            .split(hlayout[2]);

        let regs = vm.show_regs();
        let target_regs = program.get(pc).unwrap().target_regs();
        let regs_display = Paragraph::new(format_regs(&regs, target_regs))
            .block(Block::new().title("Registers").borders(Borders::ALL));
        frame.render_widget(regs_display, mem_layout[0]);

        let ram = vm.show_ram();
        let target_ram = program.get(pc).unwrap().target_ram();
        let ram_display = Paragraph::new(format_ram(vm, &ram, target_ram))
            .block(Block::new().title("RAM").borders(Borders::ALL));
        frame.render_widget(ram_display, mem_layout[1]);

        let history_height = hlayout[3].height;
        let history_display = Paragraph::new(Text::from(
            history
                .iter()
                .map(Clone::clone)
                .skip(history.len().saturating_sub(history_height as usize - 2))
                .collect::<Vec<_>>(),
        ))
        .block(Block::new().title("Output").borders(Borders::ALL));
        frame.render_widget(history_display, hlayout[3]);
    })?;

    Ok(())
}

fn format_program<'a>(
    vm: &VM,
    program: &'a [String],
    mode: bool,
    pc: usize,
    jmp: Option<(bool, uvm)>,
) -> Text<'a> {
    let mut lines = Vec::new();
    let mut spans = Vec::new();

    let load = Style::default().white().on_red();
    let exec = Style::default().white().on_green();
    let off = Style::default().black().on_dark_gray();

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("     LOAD     ", if mode { off } else { load }),
        Span::styled("     EXEC     ", if mode { exec } else { off }),
        Span::raw(" "),
    ]));

    lines.push(Line::raw(""));

    let primary = Style::default().black().on_white();
    let secondary = Style::default().black().on_dark_gray();
    let iter = program.iter().enumerate().skip(pc.saturating_sub(10));
    for (idx, str) in iter {
        let address = Span::raw(format!("\n {:08X}  ", idx));
        match jmp {
            _ if idx == pc => {
                spans.push(address);
                spans.push(Span::styled(str, primary));
            }
            Some((rfl, val)) if idx == if rfl { vm.get_reg(val) } else { val } as usize => {
                spans.push(address);
                spans.push(Span::styled(str, secondary));
            }
            _ => {
                spans.push(address);
                spans.push(Span::raw(str));
            }
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

    lines.push(Line::raw(format!(
        "            {}",
        (0..16)
            .map(|i| format!("{:02X}", i))
            .collect::<Vec<_>>()
            .join(" ")
    )));

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

        if idx % 16 == 0 {
            spans.push(Span::raw(format!("\n {:08X}   ", idx)));
        } else {
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
