
use crate::OnInput;
use crate::Platform;

fn start_game_actual<State: OnInput + Default, P: Platform<State = State>>(
    p: &mut P,
) -> Result<(), P::Error> {
    p.init()?;
    let mut state: State = p.load()?.unwrap_or_else(State::default);
    loop {
        p.draw(&state)?;
        let Some(input) = p.ask_for_input()? else {
            continue;
        };
        match state.on_input(input) {
            Some(s) => {
                state = s;
                p.save(&state)?;
            }
            None => break,
        }
    }
    Ok(())
}

pub fn start_game<State: OnInput + Default, P: Platform<State = State>>(
    p: &mut P,
) -> Result<(), P::Error> {
    let res = start_game_actual(p);
    // Whether or not the game stopped due to error or quit input, we clean up.
    let cleanup_res = p.cleanup();
    match (res, cleanup_res) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Err(_)) => Err(e), // Prefer the first error
        (Err(e), _) | (_, Err(e)) => Err(e),
    }
}
