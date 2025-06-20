use crossterm::{
    QueueableCommand,
    style::{self, Stylize},
    terminal,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};

type Pos = (i32, i32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl std::ops::Add<Dir> for Pos {
    type Output = Self;

    fn add(self, dir: Dir) -> Self::Output {
        match dir {
            Dir::Up => (self.0, self.1 - 1),
            Dir::Down => (self.0, self.1 + 1),
            Dir::Left => (self.0 - 1, self.1),
            Dir::Right => (self.0 + 1, self.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    Empty,
    Wall,
}

pub struct State {
    tiles: RefCell<HashMap<Pos, Tile>>,
    player: Pos,
}

impl State {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new().into(),
            player: (0, 0),
        }
    }

    fn generate_tile(pos: Pos) -> Tile {
        // Example logic to generate a tile based on position
        if ((412 * pos.0 + pos.1 * 313) >> 8) % 3 == 0 {
            Tile::Wall
        } else {
            Tile::Empty
        }
    }

    pub fn get_tile(&self, pos: Pos) -> Tile {
        *self
            .tiles
            .borrow_mut()
            .entry(pos)
            .or_insert(Self::generate_tile(pos))
    }

    fn on_dir_input(&mut self, dir: Dir) {
        let new_pos = self.player + dir;
        if self.get_tile(new_pos) != Tile::Wall {
            self.player = new_pos; // Move player if not hitting a wall
        }
    }

    pub fn on_input(&mut self, input: Input) {
        match input {
            Input::Up => self.on_dir_input(Dir::Up),
            Input::Down => self.on_dir_input(Dir::Down),
            Input::Left => self.on_dir_input(Dir::Left),
            Input::Right => self.on_dir_input(Dir::Right),
            Input::Quit => {
                std::io::stdout()
                    .queue(terminal::LeaveAlternateScreen)
                    .expect("Failed to leave alternate screen");
                std::process::exit(0) // Exit the game
            }
        }
    }
}

/// The chars to draw on the screen for some game thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chars {
    left: char,
    right: char,
}

impl Chars {
    pub const fn new(left: char, right: char) -> Self {
        Chars { left, right }
    }

    pub fn write(self, output: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(output, "{}{}", self.left, self.right)
    }
}

impl From<[char; 2]> for Chars {
    fn from(chars: [char; 2]) -> Self {
        Self::new(chars[0], chars[1])
    }
}

fn draw(
    state: &State,
    output: &mut impl std::io::Write,
    width: u32,
    height: u32,
) -> std::io::Result<()> {
    /// A tile get's drawn to two characters because most fonts are taller than
    /// they are wide.
    fn draw_tile(tile: Tile) -> Chars {
        match tile {
            Tile::Wall => ['█', '█'].into(),
            Tile::Empty => ['░', '░'].into(),
        }
    }
    // ░ ▒ ▓

    const TL: char = '┏';
    const T: char = '━';
    const TR: char = '┓';
    const L: char = '┃';
    const R: char = '┃';
    const BL: char = '┗';
    const B: char = '━';
    const BR: char = '┛';

    /// Player character
    const PLAYER: Chars = Chars::new('◀', '▶');

    let outer_width = width & !1 /* Ensure even */;
    let outer_height = height;
    let inner_width = outer_width - 2 /* For the frame */;
    let inner_height = outer_height - 2 /* For the frame */;
    let rows = inner_height;
    let cells_in_a_row = inner_width / 2;

    write!(output, "{}", TL)?;
    for _ in 0..inner_width {
        write!(output, "{}", T)?;
    }
    writeln!(output, "{}", TR)?;

    for row in 0..rows {
        write!(output, "{}", L)?;
        for col in 0..cells_in_a_row {
            let pos = (
                state.player.0 + col as i32 - cells_in_a_row as i32 / 2,
                state.player.1 + row as i32 - rows as i32 / 2,
            );
            let chars = if pos == state.player {
                PLAYER
            } else {
                let tile = state.get_tile(pos);
                draw_tile(tile)
            };
            chars.write(output)?;
        }
        writeln!(output, "{}", R)?;
    }

    write!(output, "{}", BL)?;
    for _ in 0..inner_width {
        write!(output, "{}", B)?;
    }
    writeln!(output, "{}", BR)?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    Quit,
}

impl TryFrom<Input> for Dir {
    type Error = ();

    fn try_from(input: Input) -> Result<Self, Self::Error> {
        match input {
            Input::Up => Ok(Dir::Up),
            Input::Down => Ok(Dir::Down),
            Input::Left => Ok(Dir::Left),
            Input::Right => Ok(Dir::Right),
            Input::Quit => Err(()),
        }
    }
}

fn get_input() -> Option<Input> {
    #[allow(clippy::unbuffered_bytes)]
    std::io::stdin()
        .bytes()
        .next()
        .and_then(|b| b.ok())
        .and_then(|b| match b as char {
            'w' => Some(Input::Up),
            's' => Some(Input::Down),
            'a' => Some(Input::Left),
            'd' => Some(Input::Right),
            'q' => Some(Input::Quit),
            _ => None,
        })
}

pub trait Platform {
    type Error;

    fn ask_for_input(&mut self) -> Result<Option<Input>, Self::Error>;
    fn draw(&mut self, state: &State) -> Result<(), Self::Error>;
}

pub fn start_game<P: Platform>(platform: &mut P) -> Result<P, P::Error> {
    let mut state = State::new();
    loop {
        platform.draw(&state)?;
        let Some(input) = platform.ask_for_input()? else {
            continue;
        };
        state.on_input(input);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CrosstermPlatform;

impl Platform for CrosstermPlatform {
    type Error = std::io::Error;
    fn ask_for_input(&mut self) -> std::io::Result<Option<Input>> {
        Ok(get_input())
    }
    fn draw(&mut self, state: &State) -> std::io::Result<()> {
        let mut out = vec![];
        draw(state, &mut out, 20, 10)?;
        std::io::stdout().write_all(&out)?;
        Ok(())
    }
}
