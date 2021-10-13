use std::cmp;
use std::io::{self, Write};
use termion::color::Rgb;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct Grid<T> {
    cells: Vec<T>,
    width: usize,
    height: usize,
}

struct Game {
    grid: Grid<Rgb>,
    active_player: usize,
    players: [Rgb; 2],
    background: Rgb,
    selection: Grid<Rgb>,
    active_selection: usize,
}

impl<T: Clone> Grid<T> {
    fn new(value: T, width: usize, height: usize) -> Self {
        let cells: Vec<T> = vec![value; width * height];
        Self {
            cells,
            width,
            height,
        }
    }
}

impl<T> Grid<T> {
    fn at(&self, x: usize, y: usize) -> &T {
        &self.cells[x * self.height + y]
    }

    fn at_mut(&mut self, x: usize, y: usize) -> &mut T {
        &mut self.cells[x * self.height + y]
    }
}

impl Game {
    pub fn new(player1_color: Rgb, player2_color: Rgb, board_color: Rgb) -> Self {
        let players = [player1_color, player2_color];
        let active_player = 0usize;
        let grid = Grid::<Rgb>::new(board_color, 7, 6);
        let active_selection = 3usize;
        let mut selection = Grid::<Rgb>::new(board_color, 7, 1);
        *selection.at_mut(active_selection, 0) = player1_color;

        Self {
            players,
            active_player,
            grid,
            background: board_color,
            selection,
            active_selection,
        }
    }

    pub fn move_selection_left(&mut self) {
        let mut new_index = self.selection.width - 1;
        if self.active_selection > 0usize {
            new_index = self.active_selection - 1;
        }
        self.move_selection(new_index);
    }

    pub fn move_selection_right(&mut self) {
        let mut new_index = 0usize;
        if self.active_selection < self.selection.width - 1 {
            new_index = self.active_selection + 1;
        }
        self.move_selection(new_index);
    }

    fn move_selection(&mut self, new_index: usize) {
        *self.selection.at_mut(self.active_selection, 0) = self.background;
        *self.selection.at_mut(new_index, 0) = self.players[self.active_player];
        self.active_selection = new_index;
    }

    pub fn drop_selected(&mut self) -> bool {
        assert!(self.can_drop_selected());

        let mut y_final = self.grid.height - 1;

        for y in 0..self.grid.height {
            if !cmp_color(self.grid.at(self.active_selection, y), &self.background) {
                y_final = y - 1;
                break;
            }
        }

        *self.grid.at_mut(self.active_selection, y_final) = self.players[self.active_player];

        self.check_winner(self.active_selection, y_final)
    }

    pub fn switch_player(&mut self) {
        self.active_player += 1;
        if self.active_player >= self.players.len() {
            self.active_player = 0;
        }
        self.move_selection(3);
    }

    pub fn can_drop_selected(&self) -> bool {
        cmp_color(self.grid.at(self.active_selection, 0), &self.background)
    }

    pub fn print_board<W: Write>(&self, stdout: &mut W) {
        print_grid(stdout, &self.selection, 0u16);
        print_grid(stdout, &self.grid, 2u16);
    }

    fn check_winner(&mut self, x: usize, y: usize) -> bool {
        let x_min = if x < 3 { 0 } else { x - 3 };
        let x_max = if x + 3 >= self.grid.width {
            self.grid.width - 1
        } else {
            x + 3
        };
        let y_min = if y < 3 { 0 } else { y - 3 };
        let y_max = if y + 3 >= self.grid.height {
            self.grid.height - 1
        } else {
            y + 3
        };

        let mut count = 0;
        let ref_color = self.grid.at(x, y);
        for i in x_min..=x_max {
            if cmp_color(self.grid.at(i, y), ref_color) {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
        }

        count = 0;
        for i in y_min..=y_max {
            if cmp_color(self.grid.at(x, i), ref_color) {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
        }

        let d_min = cmp::min(y - y_min, x - x_min);
        let d_max = cmp::min(y_max - y, x_max - x);
        count = 0;
        for i in 0..=d_max + d_min {
            if cmp_color(self.grid.at(x + i - d_min, y + i - d_min), ref_color) {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
        }

        let d_min = cmp::min(y_max - y, x - x_min);
        let d_max = cmp::min(y - y_min, x_max - x);
        count = 0;
        for i in 0..=d_max + d_min {
            if cmp_color(self.grid.at(x + i - d_min, y + d_min - i), ref_color) {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
        }

        false
    }
}

fn cmp_color(c1: &Rgb, c2: &Rgb) -> bool {
    c1.0 == c2.0 && c1.1 == c2.1 && c1.2 == c2.2
}

fn print_row<W: Write>(stdout: &mut W, cells: &Grid<Rgb>, row_index: u16, offset: u16) {
    write!(
        stdout,
        "{}{}",
        termion::cursor::Goto(1, 1 + row_index * 2 + offset),
        termion::clear::CurrentLine
    )
    .unwrap();

    for i in 0..cells.width {
        write!(
            stdout,
            "{} ● ",
            termion::color::Fg(*cells.at(i, row_index as usize))
        )
        .unwrap();
    }
}

fn print_grid<W: Write>(stdout: &mut W, cells: &Grid<Rgb>, offset: u16) {
    for i in 0..cells.height {
        print_row(stdout, cells, i as u16, offset);
    }
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().expect("Could not get stdout!");

    writeln!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
    .unwrap();

    let player1_color = Rgb(255, 0, 0);
    let player2_color = Rgb(0, 255, 0);
    let background_color = Rgb(0, 0, 0);

    let mut game = Game::new(player1_color, player2_color, background_color);
    game.print_board(&mut stdout);
    stdout.flush().unwrap();

    for c in stdin.keys() {
        match c.unwrap_or(Key::Esc) {
            Key::Left => game.move_selection_left(),
            Key::Right => game.move_selection_right(),
            Key::Down => {
                if game.can_drop_selected() {
                    if game.drop_selected() {
                        break;
                    }
                    game.switch_player();
                }
            }
            Key::Esc => break,
            _ => {}
        };
        game.print_board(&mut stdout);
        stdout.flush().unwrap();
    }

    game.print_board(&mut stdout);
    write!(stdout, "\n\r{}", termion::cursor::Show).unwrap();
}
