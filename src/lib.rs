// Std imports
use std::cell::RefCell;
use std::collections::HashMap;

// Third party
use noise::{NoiseFn, Perlin};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Some utility types.
mod utils;
use utils::{Dir, Pos};

/// Defines the kind of input that the game can receive. Input is not direct
/// keyboard and mouse presses, but a higher-level what-action-to-take kind of
/// thing. The input is keyboard presses are turned to `Input` by a `Platform`.
mod input;
pub use input::{Input, IsShift};

/// A platform is a trait that defines defines how a game interacts with the
/// local system. That includes getting input, drawing to the screen, saving and
/// loading, and whatever else there is that isn't game logic.
///
/// A `Platform` type can be used with the `start_game` function.
pub trait Platform {
    type Error;

    fn init(&mut self) -> Result<(), Self::Error>;
    fn cleanup(&mut self) -> Result<(), Self::Error>;
    fn ask_for_input(&mut self) -> Result<Option<Input>, Self::Error>;
    fn draw(&mut self, state: &State) -> Result<(), Self::Error>;
    fn save(&mut self, state: &State) -> Result<(), Self::Error>;
    fn load(&mut self) -> Result<Option<State>, Self::Error>;
}

mod game_loop;
pub use game_loop::start_game;

mod terminal_platform;
pub use terminal_platform::TerminalPlatform;

/// Defines everything to do with the tiles in the game's map.
mod tiles;
pub use tiles::Tile;

mod items;
pub use items::Item;

mod inventory;
pub use inventory::Inventory;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Menu {
    #[default]
    None,
    Inventory,
}

/// The full state of the game in any given moment.
/// This type is de/serializable for ease of the platform.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// The entire map of tiles. Serialized as a `Vec` because `toml` doesn't
    /// support non-string maps.
    ///
    /// When a position is not inside the map, it's tile will be procedurally
    /// generated. When a position is inside the map and it's tile is the same
    /// as it's procedurally generated one, it is immediately removed.
    #[serde_as(as = "RefCell<Vec<(_, _)>>")]
    tiles: RefCell<HashMap<Pos, Tile>>,
    player_pos: Pos,
    player_dir: Dir,
    // TODO: Replace this by a function.
    #[serde(default)]
    message: String,
    #[serde(default)]
    inventory: Inventory,
    #[serde(skip)]
    menu: Menu,
    #[serde(default)]
    selected_item: Option<Item>,
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
            inventory: Inventory::default(),
            menu: Menu::default(),
            selected_item: None,
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

    fn on_dir_input_no_menu(&mut self, dir: Dir, shift: IsShift) {
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
        if Tile::Empty == tile {
            if try_move {
                self.player_pos = new_pos;
            }
        } else if can_dig {
            // We are breaking the tile!
            match tile.breaks_into() {
                tiles::BreakResult::Tile(tile) => self.set_tile(new_pos, tile),
                tiles::BreakResult::Item(item) => {
                    // TODO: Save in some inventory
                    self.inventory.insert(item);
                    self.set_tile(new_pos, Tile::Empty);
                }
                tiles::BreakResult::CannotBeBroken => (),
            }
        }
    }

    fn on_dir_input_inventory(&mut self, dir: Dir, shift: IsShift) {
        let is_advancing = dir == Dir::Right || dir == Dir::Down;
        match &mut self.selected_item {
            Some(item) => *item = if is_advancing { 
                self.inventory.next(item) 
            } else { 
                self.inventory.prev(item) 
            },
            None => self.selected_item = self.inventory.first().cloned(),
        }
    }

    fn on_dir_input(&mut self, dir: Dir, shift: IsShift) {
        match self.menu {
            Menu::None => self.on_dir_input_no_menu(dir, shift),
            Menu::Inventory => self.on_dir_input_inventory(dir, shift),
        }
    }

    fn on_build(&mut self) {
        let build_pos = self.player_pos + self.player_dir;
        if self.get_tile(build_pos) != Tile::Empty {
            self.message = "You cannot build on existing tiles.".to_string();
            return; // Do not build on existing tiles
        }
        let Some(selected_item) = self.selected_item.clone() else {
            self.message = "You have no item selected to build.".to_string();
            return; // Do not build if no selected item
        };
        let Some(tile) = selected_item.to_tile() else {
            let name = selected_item.name();
            self.message = format!("You cannot build a {name}.");
            return;
        };
        self.inventory.remove(&selected_item).expect("Item should be in the inventory");
        self.set_tile(build_pos, tile);
    }

    fn tick(&mut self) {
        let tile_in_front = self.get_tile(self.player_pos + self.player_dir);
        let tile_name = tile_in_front.name();
        if self.message.is_empty() && tile_in_front != Tile::Empty {
            self.message = format!("You are facing a {tile_name}")
        }
    }

    pub fn on_input(mut self, input: Input) -> Option<Self> {
        self.message.clear();
        match input {
            Input::Dir(dir, shift) => self.on_dir_input(dir, shift),
            Input::Build => self.on_build(),
            Input::Quit => return None,
            Input::OpenInventory => self.menu = Menu::Inventory,
            Input::CloseMenu => self.menu = Menu::None,
        }
        self.tick();
        Some(self)
    }
}
