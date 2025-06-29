use crate::Input;
use crate::Platform;
use crate::State;

/// This function starts a game loop with the provided platform.
/// Returns an `Ok` when the game ended successfully (by quitting).
/// If an error that cannot be handled occurs, returns an `Err`.
pub fn start_game<P: Platform>(p: &mut P) -> Result<(), P::Error> {
    // This function wraps the function below, and just gracefully handles
    // errors.
    let res = start_game_actual(p);
    // Whether or not the game stopped due to error or quit input, we clean up.
    let cleanup_res = p.cleanup();
    match (res, cleanup_res) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Err(_)) => Err(e), // Prefer the first error
        (Err(e), _) | (_, Err(e)) => Err(e),
    }
}

fn get_good_input<P: Platform>(p: &mut P) -> Result<Input, P::Error> {
    loop {
        match p.ask_for_input()? {
            Some(input) => return Ok(input),
            None => continue, // No input, try again
        }
    }
}

fn start_game_actual<P: Platform>(p: &mut P) -> Result<(), P::Error> {
    p.init()?;
    let mut state = p.load()?.unwrap_or_else(State::new);
    loop {
        p.draw(&state)?;
        let input = get_good_input(p)?;
        match state.on_input(input) {
            Some(s) => {
                state = s;
                p.save(&state)?;
            }
            None => break,
        }
    }
    // No call to `p.cleanup()`, the calling function calls it.
    Ok(())
}
