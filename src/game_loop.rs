use std::path::Path;

use crate::Platform;
use crate::State;

fn start_game_actual<P: Platform>(p: &mut P) -> Result<(), P::Error> {
    p.init()?;
    let save_path = &Path::new("save");
    let backup_path = &Path::new("backup");
    let mut state: State = p.read(save_path)?.unwrap_or_else(State::new);
    loop {
        p.write(backup_path, &state)?;
        p.draw(&state)?;
        let Some(input) = p.ask_for_input()? else { continue };
        match state.on_input(input) {
            Some(s) => {
                state = s;
                p.write(save_path, &state)?;
            },
            None => break,
        }
    }
    Ok(())
}

pub fn start_game<P: Platform>(p: &mut P) -> Result<(), P::Error> {
    let res = start_game_actual(p);
    // Whether or not the game stopped due to error or quit input, we clean up.
    let cleanup_res = p.cleanup();
    match (res, cleanup_res) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Err(_)) => Err(e), // Prefer the first error
        (Err(e), _) | (_, Err(e)) => Err(e),
    }
}
