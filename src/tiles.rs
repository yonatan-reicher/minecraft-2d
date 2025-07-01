use crate::Item;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    WallFull,
    WallHalf,
    WallLow,
    Wood(u8),
}

/// What does a tile break into?
pub enum BreakResult {
    Tile(Tile),
    Item(Item),
    CannotBeBroken,
}

impl From<Tile> for BreakResult {
    fn from(tile: Tile) -> BreakResult {
        Self::Tile(tile)
    }
}

impl From<Item> for BreakResult {
    fn from(item: Item) -> BreakResult {
        Self::Item(item)
    }
}

impl Tile {
    /// What does this tile break into?
    pub fn breaks_into(self) -> BreakResult {
        match self {
            Tile::WallFull => Tile::WallHalf.into(),
            Tile::WallHalf => Tile::WallLow.into(),
            Tile::WallLow => Item::Wall.into(),
            Tile::Empty => BreakResult::CannotBeBroken,
            Tile::Wood(0) => Item::Wood.into(),
            Tile::Wood(n) => Tile::Wood(n - 1).into(),
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Tile::Empty => "empty",
            Tile::WallFull => "wall",
            Tile::WallHalf => "broken wall",
            Tile::WallLow => "very broken wall",
            Tile::Wood(_) => "wood",
        }
    }
}

