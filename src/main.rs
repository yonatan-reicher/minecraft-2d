use minecraft_2d::*;

fn main() {
    start_game(&mut TerminalPlatform)
        .unwrap_or_else(|e| eprintln!("Error: {}", e));
}
