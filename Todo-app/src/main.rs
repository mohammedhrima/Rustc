use ncurses::*;
use std::fs::File;
use std::io::{self, Write, BufRead};
use std::env;
use std::process;

const RECULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

type Id = usize;

#[derive(Default)]
struct Ui {
    list_curr: Option<Id>,
    row: usize,
    col: usize,
}

impl Ui {
    fn begin(&mut self, row: usize, col: usize) {
        clear();
        self.row = row;
        self.col = col;
    }
    fn end(&mut self) {}
    fn begin_list(&mut self, id: Id) {
        assert!(self.list_curr.is_none(), "Nested list are not allowed");
        self.list_curr = Some(id);
    }
    fn list_element(&mut self, label: &str, id: Id) -> bool {
        let id_curr = self
            .list_curr
            .expect("Not allowed to create lists element outside of lists");
        self.label(label, {
            if id_curr == id {
                HIGHLIGHT_PAIR
            } else {
                RECULAR_PAIR
            }
        });
        return false;
    }
    fn label(&mut self, text: &str, pair: i16) {
        mv(self.row as i32, self.col as i32);
        attron(COLOR_PAIR(pair));
        let _ = addstr(text);
        attroff(COLOR_PAIR(pair));
        self.row += 1;
    }
    fn end_list(&mut self) {
        // todo!();
        self.list_curr = None;
    }
}

#[derive(Debug)]
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

fn list_up(list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr -= 1;
    }
}

fn list_down(list: &Vec<String>, list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        *list_curr += 1;
    }
}

fn list_transfer(list_dst: &mut Vec<String>, list_src: &mut Vec<String>, list_src_curr: &mut usize) {
    if *list_src_curr < list_src.len() {
        list_dst.push(list_src.remove(*list_src_curr));
        if *list_src_curr >= list_src.len() && list_src.len() > 0 {
            *list_src_curr = list_src.len() - 1;
        }
    }

}

fn parse_item(line: &str) -> Option<(Status, &str)> {
    let todo_prefix = "TODO: ";
    let done_prefix = "DONE: ";
    if line.starts_with(todo_prefix) {
        return Some((Status::Todo, &line[todo_prefix.len()..]))
    }
    if line.starts_with(done_prefix) {
        return Some((Status::Done, &line[done_prefix.len()..]))
    }
    return None;
}

fn load_state(todos: &mut Vec <String>, dones: &mut Vec <String>, file_path: &str)
{
    let file = File::open(file_path).unwrap();
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        match parse_item(&line.unwrap()) {
            Some((Status::Todo, title)) => todos.push(title.to_string()),
            Some((Status::Done, title)) => dones.push(title.to_string()),
            None => {
                eprintln!("{}:{}: Error: bad format", file_path, index + 1);
                process::exit(1);
            }
        }
    }
}

fn save_state(todos: &Vec<String>, dones: &Vec<String>, file_path: &str)
{
    let mut file = File::create(file_path).unwrap();
    for todo in todos.iter() {
        let _ = writeln!(file, "TODO: {}", todo);
    }
    for done in dones.iter() {
        let _ = writeln!(file, "DONE: {}", done);
    }
}

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    let file_path = match args.next() {
            Some(file_path) => file_path,
            None => "TODO.md".to_string()
        };
    let mut todos = Vec::<String>::new();
    let mut dones = Vec::<String>::new();
    let mut todo_curr: usize = 0;
    let mut done_curr: usize = 0;
    load_state(&mut todos, &mut dones, &file_path);

    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(RECULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    let mut quit = false;
    let mut focus = Status::Todo;

    let mut ui = Ui::default();
    while !quit {
        ui.begin(0, 0);
        {
            match focus {
                Status::Todo => {
                    ui.label("[TODO] DONE ", RECULAR_PAIR);
                    ui.label("------------", RECULAR_PAIR);
                    ui.begin_list(todo_curr);
                    for (index, todo) in todos.iter().enumerate() {
                        ui.list_element(&format!("- [ ] {}", todo), index);
                    }
                    ui.end_list();
                }
                Status::Done => {
                    ui.label(" TODO [DONE]", RECULAR_PAIR);
                    ui.label("------------", RECULAR_PAIR);
                    ui.begin_list(done_curr);
                    for (index, done) in dones.iter().enumerate() {
                        ui.list_element(&format!("- [x] {}", done), index);
                    }
                    ui.end_list();
                }
            }
        }
        ui.end();
        refresh();

        let key = getch();
        match key as u8 as char {
            'q' => quit = true,
            's' => save_state(&todos, &dones, &file_path),
            'A' => match focus {
                Status::Todo => list_up(&mut todo_curr),
                Status::Done => list_up(&mut done_curr),
            },
            'B' => match focus {
                Status::Todo => list_down(&todos, &mut todo_curr),
                Status::Done => list_down(&dones, &mut done_curr),
            },
            '\n' => match focus {
                Status::Todo => list_transfer(&mut dones, &mut todos, &mut todo_curr),
                Status::Done => list_transfer(&mut todos, &mut dones, &mut done_curr),
            },
            '\t' => {
                focus = focus.toggle();
            }
            _ => {
                // todos.push(format!("{}", key));
            }
        }
    }
    
    endwin();
}
