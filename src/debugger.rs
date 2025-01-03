use crate::{instruction::Instruction, loader, uvm, vm::VM, REG_LEN};
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

#[derive(Default)]
struct DisplayState {
    pc: usize,
    ram_offset: usize,
    program_offset: usize,
}

pub fn run(program: &[u8]) -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = start(terminal, program);
    ratatui::restore();
    app_result
}

fn start(mut terminal: DefaultTerminal, program: &[u8]) -> io::Result<()> {
    let mut vm = VM::new();
    let program_end = vm.load(program) as usize;
    let mut next_instruction = None;
    let mut last_instruction = loader::decode(program, 0).expect("Invalid program start");
    let mut display_state = DisplayState::default();
    let mut auto = false;
    let mut done = false;
    let mut history = Vec::new();
    let display_program = vm
        .show_program()
        .into_iter()
        .take_while(|(_, n)| *n < program_end)
        .collect::<Vec<_>>();

    load_next(&vm, &mut next_instruction, &mut display_state);

    loop {
        draw(
            &mut terminal,
            next_instruction.is_some(),
            &vm,
            &display_program,
            last_instruction,
            &mut display_state,
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
                            vm.load(program);
                            history = Vec::new();
                            load_next(&vm, &mut next_instruction, &mut display_state);
                        }
                        KeyCode::Enter => auto = !auto,
                        KeyCode::Char(' ') => {
                            auto = false;
                            if done {
                                continue;
                            }
                            if let Some(instruction) = next_instruction {
                                execute(&mut vm, instruction, &mut done, &mut auto, &mut history);
                                next_instruction = None;
                            } else {
                                load_next(&vm, &mut next_instruction, &mut display_state);
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
            load_next(&vm, &mut next_instruction, &mut display_state);
            if let Some(instruction) = next_instruction {
                execute(&mut vm, instruction, &mut done, &mut auto, &mut history);
            }
        }

        vm.stdout()
            .lines()
            .for_each(|l| history.push(Line::raw(l.to_owned())));

        vm.stderr()
            .lines()
            .for_each(|l| history.push(Line::raw(l.to_owned()).yellow()));

        if let Some(instruction) = next_instruction {
            last_instruction = instruction;
        }
    }
}

fn execute(
    vm: &mut VM,
    instruction: Instruction,
    done: &mut bool,
    auto: &mut bool,
    history: &mut Vec<Line<'_>>,
) {
    match vm.execute(instruction) {
        Ok(Some(exit_code)) => {
            history.push(Line::raw(format!("Program exited with code : {exit_code}")));
            *done = true;
        }
        Ok(None) => (),
        Err(err) => {
            history.push(Line::raw(err).red());
            *auto = false;
        }
    }
}

fn load_next(
    vm: &VM,
    next_instruction: &mut Option<Instruction>,
    display_state: &mut DisplayState,
) {
    if let Some(instruction) = vm.decode() {
        *next_instruction = Some(instruction);
        display_state.pc = vm.pc() as usize;
    }
}

#[allow(clippy::indexing_slicing)]
fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mode: bool,
    vm: &VM,
    program: &[(Instruction, usize)],
    instruction: Instruction,
    display_state: &mut DisplayState,
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

        let program_str = program
            .iter()
            .map(|(instr, addr)| (format!("{instr:?}"), *addr))
            .collect::<Vec<_>>();
        let program_jmp = instruction.target_addr();
        let program_display = Paragraph::new(format_program(
            vm,
            &program_str,
            mode,
            hlayout[1].height,
            display_state,
            program_jmp,
        ))
        .block(Block::new().title("Program").borders(Borders::ALL));
        frame.render_widget(program_display, hlayout[1]);

        let mem_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(6), Constraint::Fill(1)])
            .split(hlayout[2]);

        match vm.show_regs() {
            Ok(regs) => {
                let target_regs = instruction.target_regs();
                frame.render_widget(
                    Paragraph::new(format_regs(&regs, target_regs))
                        .block(Block::new().title("Registers").borders(Borders::ALL)),
                    mem_layout[0],
                );
            }
            Err(err) => frame.render_widget(
                Paragraph::new(format!("CORRUPTED: {err}").red())
                    .block(Block::new().title("Registers").borders(Borders::ALL)),
                mem_layout[0],
            ),
        }

        let ram = vm.show_ram();
        let target_ram = instruction.target_ram();
        let ram_display = Paragraph::new(format_ram(
            vm,
            &ram,
            &target_ram,
            mem_layout[1].height,
            display_state,
        ))
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
        .block(
            Block::new()
                .title("stdout")
                .title("stderr".yellow())
                .borders(Borders::ALL),
        );
        frame.render_widget(history_display, hlayout[3]);
    })?;

    Ok(())
}

fn format_program<'a>(
    vm: &VM,
    program: &'a [(String, usize)],
    mode: bool,
    height: u16,
    display_state: &mut DisplayState,
    jmp: Option<(bool, uvm)>,
) -> Text<'a> {
    let height: usize = (height - 4).into();
    if let Some(idx) = program
        .iter()
        .enumerate()
        .find(|(_, (_, addr))| *addr == display_state.pc)
        .map(|x| x.0)
    {
        display_state.program_offset = idx
            .saturating_sub((height - 1) / 2)
            .min(program.len().saturating_sub(height));
    }

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
    for (str, addr) in program.iter().skip(display_state.program_offset) {
        let address = Span::raw(format!("\n {addr:08X}  "));
        match jmp {
            _ if *addr == display_state.pc => {
                spans.push(address);
                spans.push(Span::styled(str, primary));
            }
            Some((rfl, val)) if rfl && vm.get_reg(val).is_err() => {
                spans.push(address);
                spans.push(Span::raw(str));
            }
            Some((rfl, val)) if *addr == if rfl { vm.get_reg(val).unwrap() } else { val } as usize => {
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
fn format_ram<'a>(
    vm: &VM,
    ram: &'a [String],
    targets: &[(bool, uvm, bool)],
    height: u16,
    display_state: &mut DisplayState,
) -> Text<'a> {
    let height: usize = (height - 3).into();
    if let Some((rfl, val, _)) = targets.first() {
        if let Ok(target) = if *rfl { vm.get_reg(*val) } else { Ok(*val) } {
            let target = target as usize;
            let mut offset = target.saturating_sub((height * 16) / 2);
            offset = offset.saturating_sub(offset % 16);
            display_state.ram_offset = offset;
        }
    }

    let mut lines = Vec::new();

    lines.push(Line::raw(format!(
        "            {}",
        (0..16)
            .map(|i| format!("{i:02X}"))
            .collect::<Vec<_>>()
            .join(" ")
    )));

    let write_style = Style::default().black().on_white();
    let read_style = Style::default().black().on_dark_gray();
    let mut keep_highlighting = 0;
    let mut spans = Vec::new();
    let mut current_style = Style::default();
    for (idx, str) in ram.iter().enumerate().skip(display_state.ram_offset) {
        let mut style = if keep_highlighting > 0 {
            keep_highlighting -= 1;
            current_style
        } else {
            Style::default()
        };

        if idx % 16 == 0 {
            spans.push(Span::raw(format!("\n {idx:08X}   ")));
        } else {
            spans.push(Span::styled(" ", style));
        }

        if let Some((_, _, write)) = targets.iter().find(|(rfl, val, _)| {
            idx == if *rfl {
                match vm.get_reg(*val) {
                    Ok(a) => a,
                    Err(_) => return false,
                }
            } else {
                *val
            } as usize
        }) {
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
