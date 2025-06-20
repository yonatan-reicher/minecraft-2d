use crate::Input;
use crate::Platform;
use crate::State;
use crate::Tile;
use crossterm::execute;
use crossterm::queue;
use crossterm::terminal;
use std::io::Read;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;

fn get_input() -> Option<Input> {
    #[allow(clippy::unbuffered_bytes)]
    stdin()
        .bytes()
        .next()
        .and_then(|b| b.ok())
        .and_then(|b| match b as char {
            'w' => Some(Input::Up),
            's' => Some(Input::Down),
            'a' => Some(Input::Left),
            'd' => Some(Input::Right),
            'q' => Some(Input::Quit),
            _ => None,
        })
}

/// The chars to draw on the screen for some game thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chars {
    left: char,
    right: char,
}

impl Chars {
    pub const fn new(left: char, right: char) -> Self {
        Chars { left, right }
    }

    pub fn write(self, output: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(output, "{}{}", self.left, self.right)
    }
}

impl From<[char; 2]> for Chars {
    fn from(chars: [char; 2]) -> Self {
        Self::new(chars[0], chars[1])
    }
}

fn draw(
    state: &State,
    output: &mut impl std::io::Write,
    width: u32,
    height: u32,
) -> std::io::Result<()> {
    /// A tile get's drawn to two characters because most fonts are taller than
    /// they are wide.
    fn draw_tile(tile: Tile) -> Chars {
        match tile {
            Tile::WallFull => ['█', '█'].into(),
            Tile::WallHalf => ['▓', '▓'].into(),
            Tile::WallLow => ['▒', '▒'].into(),
            Tile::Empty => [' ', ' '].into(),
        }
    }
    // ░ ▒ ▓

    const TL: char = '┏';
    const T: char = '━';
    const TR: char = '┓';
    const L: char = '┃';
    const R: char = '┃';
    const BL: char = '┗';
    const B: char = '━';
    const BR: char = '┛';

    /// Player character
    const PLAYER: Chars = Chars::new('(', ')');

    let outer_width = width & !1 /* Ensure even */;
    let outer_height = height;
    let inner_width = outer_width - 2 /* For the frame */;
    let inner_height = outer_height - 2 /* For the frame */;
    let rows = inner_height;
    let cells_in_a_row = inner_width / 2;

    write!(output, "{}", TL)?;
    for _ in 0..inner_width {
        write!(output, "{}", T)?;
    }
    writeln!(output, "{}", TR)?;

    for row in 0..rows {
        write!(output, "{}", L)?;
        for col in 0..cells_in_a_row {
            let pos = (
                state.player.0 + col as i32 - cells_in_a_row as i32 / 2,
                state.player.1 + row as i32 - rows as i32 / 2,
            );
            let chars = if pos == state.player {
                PLAYER
            } else {
                let tile = state.get_tile(pos);
                draw_tile(tile)
            };
            chars.write(output)?;
        }
        writeln!(output, "{}", R)?;
    }

    write!(output, "{}", BL)?;
    for _ in 0..inner_width {
        write!(output, "{}", B)?;
    }
    writeln!(output, "{}", BR)?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerminalPlatform;

impl Platform for TerminalPlatform {
    type Error = std::io::Error;

    fn init(&mut self) -> std::io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        Ok(())
    }

    fn cleanup(&mut self) -> std::io::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(stdout(), terminal::LeaveAlternateScreen)?;
        Ok(())
    }

    fn ask_for_input(&mut self) -> std::io::Result<Option<Input>> {
        Ok(get_input())
    }

    fn draw(&mut self, state: &State) -> std::io::Result<()> {
        queue!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        let mut out = vec![];
        let (w, h) = terminal::size()?;
        draw(state, &mut out, w as _, (h - 1) as _)?;
        std::io::stdout().write_all(&out)?;
        Ok(())
    }
}
