use crate::Tile;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    Wall,
}

impl Item {
    pub fn name(&self) -> String {
        match self {
            Item::Wall => "wall".into(),
        }
    }

    pub fn to_tile(&self) -> Option<Tile> {
        match self {
            Item::Wall => Some(Tile::WallFull),
        }
    }
}
