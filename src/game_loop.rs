use crate::Platform;
use crate::State;

pub enum Never {}

pub fn start_game<P: Platform>(p: &mut P) -> Result<Never, P::Error> {
    let mut state = State::new();
    loop {
        p.draw(&state)?;
        if let Some(input) = p.ask_for_input()? {
            state.on_input(input);
        }
    }
}
