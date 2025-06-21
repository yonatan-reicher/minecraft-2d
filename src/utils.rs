pub type Pos = (i32, i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dir {
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

