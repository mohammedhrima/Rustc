use std::env;
use std::fs::File;
use std::io::{self, BufRead, Stdout, Write};
use std::process;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    style::{Attribute, Print, SetAttribute},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};

enum Status {
    Todo,
    Done,
}

impl Status {
    fn toggle(&self) -> Self {
        match self {
            Status::Todo => Status::Done,
            Status::Done => Status::Todo,
        }
    }
}

fn parse_item(line: &str) -> Option<(Status, &str)> {
    if let Some(rest) = line.strip_prefix("TODO: ") {
        return Some((Status::Todo, rest));
    }
    if let Some(rest) = line.strip_prefix("DONE: ") {
        return Some((Status::Done, rest));
    }
    None
}

fn load_state(todos: &mut Vec<String>, dones: &mut Vec<String>, file_path: &str) {
    let file = File::open(file_path).unwrap_or_else(|err| {
        eprintln!("error: could not open {file_path}: {err}");
        process::exit(1);
    });
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        let line = line.unwrap();
        match parse_item(&line) {
            Some((Status::Todo, title)) => todos.push(title.to_string()),
            Some((Status::Done, title)) => dones.push(title.to_string()),
            None => {
                eprintln!("{file_path}:{}: error: expected a 'TODO: ' or 'DONE: ' line", index + 1);
                process::exit(1);
            }
        }
    }
}

fn save_state(todos: &[String], dones: &[String], file_path: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    for todo in todos {
        writeln!(file, "TODO: {todo}")?;
    }
    for done in dones {
        writeln!(file, "DONE: {done}")?;
    }
    Ok(())
}

fn list_up(curr: &mut usize) {
    if *curr > 0 {
        *curr -= 1;
    }
}

fn list_down(list: &[String], curr: &mut usize) {
    if *curr + 1 < list.len() {
        *curr += 1;
    }
}

fn list_transfer(dst: &mut Vec<String>, src: &mut Vec<String>, src_curr: &mut usize) {
    if *src_curr < src.len() {
        dst.push(src.remove(*src_curr));
        if *src_curr >= src.len() && !src.is_empty() {
            *src_curr = src.len() - 1;
        }
    }
}

fn draw_line(out: &mut Stdout, row: &mut u16, text: &str, highlight: bool) -> io::Result<()> {
    out.queue(cursor::MoveTo(0, *row))?;
    if highlight {
        out.queue(SetAttribute(Attribute::Reverse))?;
    }
    out.queue(Print(text))?;
    if highlight {
        out.queue(SetAttribute(Attribute::Reset))?;
    }
    *row += 1;
    Ok(())
}

fn render(
    out: &mut Stdout,
    todos: &[String],
    dones: &[String],
    focus: &Status,
    todo_curr: usize,
    done_curr: usize,
) -> io::Result<()> {
    out.queue(Clear(ClearType::All))?;
    let (header, items, curr, mark) = match focus {
        Status::Todo => ("[TODO] DONE", todos, todo_curr, "- [ ] "),
        Status::Done => (" TODO [DONE]", dones, done_curr, "- [x] "),
    };
    let mut row = 0;
    draw_line(out, &mut row, header, false)?;
    draw_line(out, &mut row, "------------", false)?;
    for (index, item) in items.iter().enumerate() {
        draw_line(out, &mut row, &format!("{mark}{item}"), index == curr)?;
    }
    row += 1;
    draw_line(out, &mut row, "[up/down] move  [enter] toggle  [tab] switch  [s] save  [q] quit", false)?;
    out.flush()
}

fn run(out: &mut Stdout, file_path: &str) -> io::Result<()> {
    let mut todos = Vec::new();
    let mut dones = Vec::new();
    let mut todo_curr = 0;
    let mut done_curr = 0;
    load_state(&mut todos, &mut dones, file_path);

    let mut focus = Status::Todo;
    loop {
        render(out, &todos, &dones, &focus, todo_curr, done_curr)?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') => break,
            KeyCode::Char('s') => save_state(&todos, &dones, file_path)?,
            KeyCode::Up => match focus {
                Status::Todo => list_up(&mut todo_curr),
                Status::Done => list_up(&mut done_curr),
            },
            KeyCode::Down => match focus {
                Status::Todo => list_down(&todos, &mut todo_curr),
                Status::Done => list_down(&dones, &mut done_curr),
            },
            KeyCode::Enter => match focus {
                Status::Todo => list_transfer(&mut dones, &mut todos, &mut todo_curr),
                Status::Done => list_transfer(&mut todos, &mut dones, &mut done_curr),
            },
            KeyCode::Tab => focus = focus.toggle(),
            _ => {}
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let file_path = env::args().nth(1).unwrap_or_else(|| "TODO.md".to_string());

    let mut out = io::stdout();
    terminal::enable_raw_mode()?;
    out.queue(EnterAlternateScreen)?.queue(cursor::Hide)?;
    out.flush()?;

    let result = run(&mut out, &file_path);

    out.queue(cursor::Show)?.queue(LeaveAlternateScreen)?;
    out.flush()?;
    terminal::disable_raw_mode()?;
    result
}
