use noise::NoiseFn;
use noise::Perlin;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

mod utils;
use utils::{Dir, Pos};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    WallFull,
    WallHalf,
    WallLow,
}

impl Tile {
    pub fn breaks_into(self) -> Option<Tile> {
        match self {
            Tile::WallFull => Some(Tile::WallHalf),
            Tile::WallHalf => Some(Tile::WallLow),
            Tile::WallLow => Some(Tile::Empty),
            Tile::Empty => None,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde_as(as = "RefCell<Vec<(_, _)>>")]
    tiles: RefCell<HashMap<Pos, Tile>>,
    player_pos: Pos,
    player_dir: Dir,
    #[serde(default)]
    message: String,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new().into(),
            player_pos: (0, 0),
            player_dir: Dir::Down,
            message: String::new(),
        }
    }

    fn generate_tile(pos: Pos) -> Tile {
        let f = Perlin::new(12412).get([pos.0 as f64 * 0.1, pos.1 as f64 * 0.1]);
        // now `f` is a value between -1.0 and 1.0
        let f = (f + 1.0) / 2.0; // normalize to [0.0, 1.0]
        if f < 0.3 { Tile::WallFull } else { Tile::Empty }
    }

    pub fn get_tile(&self, pos: Pos) -> Tile {
        self.tiles
            .borrow()
            .get(&pos)
            .cloned()
            .unwrap_or_else(|| Self::generate_tile(pos))
    }

    pub fn set_tile(&mut self, pos: Pos, tile: Tile) {
        let mut tiles = self.tiles.borrow_mut();
        if tile == Self::generate_tile(pos) {
            tiles.remove(&pos);
        } else {
            tiles.insert(pos, tile);
        }
    }

    fn on_dir_input(&mut self, dir: Dir, shift: IsShift) {
        let dir_same = self.player_dir == dir;

        if shift == IsShift::No {
            self.player_dir = dir;
        };

        let new_pos = self.player_pos + dir;
        let try_move = match shift {
            IsShift::Yes => true,
            IsShift::No => dir_same,
        };
        let can_dig = dir_same;
        let tile = self.get_tile(new_pos);
        match tile {
            Tile::Empty => {
                if try_move {
                    self.player_pos = new_pos
                }
            }
            Tile::WallFull => {
                if can_dig {
                    self.set_tile(new_pos, tile.breaks_into().unwrap())
                }
            }
            Tile::WallHalf => {
                if can_dig {
                    self.set_tile(new_pos, tile.breaks_into().unwrap())
                }
            }
            Tile::WallLow => {
                if can_dig {
                    self.set_tile(new_pos, tile.breaks_into().unwrap())
                }
            }
        }
    }

    fn on_build(&mut self) {
        let build_pos = self.player_pos + self.player_dir;
        if self.get_tile(build_pos) != Tile::Empty {
            return; // Do not build on existing tiles
        }
        self.set_tile(build_pos, Tile::WallFull);
    }

    fn tick(&mut self) {
        let tile_in_front = self.get_tile(self.player_pos + self.player_dir);
        match tile_in_front {
            Tile::Empty => self.message.clear(),
            Tile::WallFull => self.message = "You are facing a full wall.".to_string(),
            Tile::WallHalf => self.message = "You are facing a half wall.".to_string(),
            Tile::WallLow => self.message = "You are facing a low wall.".to_string(),
        }
    }

    pub fn on_input(mut self, input: Input) -> Option<Self> {
        match input {
            Input::Dir(dir, shift) => self.on_dir_input(dir, shift),
            Input::Build => self.on_build(),
            Input::Quit => return None,
        }
        self.tick();
        Some(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsShift {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Dir(Dir, IsShift),
    Build,
    Quit,
}

impl TryFrom<Input> for Dir {
    type Error = ();

    fn try_from(input: Input) -> Result<Self, Self::Error> {
        match input {
            Input::Dir(dir, _) => Ok(dir),
            Input::Build => Err(()),
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
    fn read<T: DeserializeOwned>(&mut self, file_path: &Path) -> Result<Option<T>, Self::Error>;
    fn write<T: serde::Serialize>(&mut self, file_path: &Path, value: T)
    -> Result<(), Self::Error>;
}

mod game_loop;
pub use game_loop::start_game;
mod terminal_platform;
pub use terminal_platform::TerminalPlatform;
