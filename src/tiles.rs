use serde::{Deserialize, Serialize};

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

