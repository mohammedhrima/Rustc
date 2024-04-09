extern crate ncurses;
use ncurses::*;

const REGULAR_PAIR:i16 = 0;
const HIGHTLIGHTED_PAIR: i16 = 1;

fn main() {
    initscr();
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHTLIGHTED_PAIR, COLOR_BLACK, COLOR_WHITE);

    let mut quit = false;
    let todos = vec![
        "task 1",
        "task 2",
        "task 3",
        "task 4"
    ];
    let curr : usize = 0;

    while !quit {
        for (index, todo) in todos.iter().enumerate() {
            let pair:i16 = if curr == index {
                    HIGHTLIGHTED_PAIR;
                }
                else {
                    REGULAR_PAIR;
                };
            attron(COLOR_PAIR(pair));
            mvprintw(index as i32, 1, todo);
            attroff(COLOR_PAIR(pair));
        };
        refresh();

        let key = getch();
        match key as u8 as char {
            'q' => quit = true,
            _ => {}
        }
    }
    getch();
    endwin();
}