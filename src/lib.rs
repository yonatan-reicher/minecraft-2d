use std::cell::RefCell;
use std::collections::HashMap;

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
pub enum Tile {
    Empty,
    Wall,
}

#[derive(Debug, Default, Clone)]
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

    pub fn on_input(mut self, input: Input) -> Option<Self> {
        match input {
            Input::Up => self.on_dir_input(Dir::Up),
            Input::Down => self.on_dir_input(Dir::Down),
            Input::Left => self.on_dir_input(Dir::Left),
            Input::Right => self.on_dir_input(Dir::Right),
            Input::Quit => return None,
        }
        Some(self)
    }
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

pub trait Platform {
    type Error;
    fn init(&mut self) -> Result<(), Self::Error>;
    fn cleanup(&mut self) -> Result<(), Self::Error>;
    fn ask_for_input(&mut self) -> Result<Option<Input>, Self::Error>;
    fn draw(&mut self, state: &State) -> Result<(), Self::Error>;
}

mod game_loop;
pub use game_loop::start_game;
mod terminal_platform;
pub use terminal_platform::TerminalPlatform;
