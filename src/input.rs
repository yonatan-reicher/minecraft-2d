use crate::utils::Dir;

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
