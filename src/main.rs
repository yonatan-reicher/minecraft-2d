use minecraft_2d::*;
use std::io::Write;
use crossterm::QueueableCommand;
use crossterm::terminal;

fn main() {
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    std::io::stdout()
        .queue(terminal::EnterAlternateScreen)
        .expect("Failed to enter alternate screen");
    start_game(&mut CrosstermPlatform).unwrap();
}
