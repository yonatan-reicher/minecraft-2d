use crate::Dir;
use crate::Input;
use crate::IsShift;
use crate::Platform;
use crate::State;
use crate::Tile;
use crossterm::cursor;
use crossterm::execute;
use crossterm::queue;
use crossterm::style;
use crossterm::style::Color;
use crossterm::style::Colors;
use crossterm::style::Print;
use crossterm::terminal;
use std::io;
use std::io::Read;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;

fn data_dir() -> io::Result<std::path::PathBuf> {
    // TODO: Maybe return a result?
    let dir = dirs::data_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find data directory"))?;
    let out = dir.join("j-minecraft-2d");
    if !out.exists() {
        std::fs::create_dir_all(&out)?;
    }
    Ok(out)
}

fn get_input() -> Option<Input> {
    // TODO: Currently, this buffers input. So if you spam a key, it will keep
    // being registered as pressed even after you let go of the button (if there
    // is some lag). To avoid this, we want another thread reading input and
    // blocking, and sending them individually, but to a 1-length buffer.
    #[allow(clippy::unbuffered_bytes)]
    stdin()
        .bytes()
        .next()
        .and_then(|b| b.ok())
        .and_then(|b| match b as char {
            'w' => Some(Input::Dir(Dir::Up, IsShift::No)),
            's' => Some(Input::Dir(Dir::Down, IsShift::No)),
            'a' => Some(Input::Dir(Dir::Left, IsShift::No)),
            'd' => Some(Input::Dir(Dir::Right, IsShift::No)),
            'W' => Some(Input::Dir(Dir::Up, IsShift::Yes)),
            'S' => Some(Input::Dir(Dir::Down, IsShift::Yes)),
            'A' => Some(Input::Dir(Dir::Left, IsShift::Yes)),
            'D' => Some(Input::Dir(Dir::Right, IsShift::Yes)),
            'b' | 'B' => Some(Input::Build),
            'q' => Some(Input::Quit),
            _ => None,
        })
}

/// The chars to draw on the screen for some game thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chars {
    left: char,
    right: char,
    bg: Color,
    fg: Color,
}

impl Chars {
    pub const fn new(left: char, right: char) -> Self {
        Chars {
            left,
            right,
            bg: Color::Reset,
            fg: Color::Reset,
        }
    }

    pub const fn with_fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }

    pub const fn with_bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }

    pub fn write(self, output: &mut impl io::Write) -> io::Result<()> {
        queue!(
            output,
            style::SetColors(Colors::new(self.fg, self.bg)),
            Print(self.left),
            Print(self.right),
        )
    }
}

impl From<[char; 2]> for Chars {
    fn from(chars: [char; 2]) -> Self {
        Self::new(chars[0], chars[1])
    }
}

fn draw(state: &State, output: &mut impl io::Write, width: u32, height: u32) -> io::Result<()> {
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
    fn player(dir: Dir) -> Chars {
        Chars::from(match dir {
            Dir::Up => ['▀', '▀'],
            Dir::Down => ['▄', '▄'],
            Dir::Left => ['█', ' '],
            Dir::Right => [' ', '█'],
        })
        .with_fg(Color::White)
        .with_bg(Color::DarkGrey)
    }

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
                state.player_pos.0 + col as i32 - cells_in_a_row as i32 / 2,
                state.player_pos.1 + row as i32 - rows as i32 / 2,
            );
            let chars = if pos == state.player_pos {
                player(state.player_dir)
            } else {
                let tile = state.get_tile(pos);
                queue!(output, style::SetForegroundColor(style::Color::Green))?;
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

#[derive(Debug)]
enum Error {
    Ser(std::path::PathBuf, toml::ser::Error),
    De(std::path::PathBuf, toml::de::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Ser(path, e) => write!(f, "Failed to serialize to {}: {}", path.display(), e),
            Error::De(path, e) => write!(f, "Failed to deserialize from {}: {}", path.display(), e),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerminalPlatform;

impl Platform for TerminalPlatform {
    type Error = io::Error;

    fn init(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(stdout(), terminal::LeaveAlternateScreen)?;
        Ok(())
    }

    fn ask_for_input(&mut self) -> io::Result<Option<Input>> {
        Ok(get_input())
    }

    fn draw(&mut self, state: &State) -> io::Result<()> {
        queue!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )?;
        let mut out = vec![];
        let (w, h) = terminal::size()?;
        draw(state, &mut out, w as _, (h - 1) as _)?;
        io::stdout().write_all(&out)?;
        execute!(
            stdout(),
            cursor::MoveTo(1, 1),
            Print(HELP[0]),
            cursor::MoveTo(1, 2),
            Print(HELP[1]),
            cursor::MoveTo(1, 3),
            Print(HELP[2]),
            cursor::MoveTo(1, 4),
            Print(HELP[3]),
            cursor::MoveTo(1, 5),
            Print(HELP[4]),
        )?;
        Ok(())
    }

    fn read<T: serde::de::DeserializeOwned>(
        &mut self,
        file_path: &std::path::Path,
    ) -> Result<Option<T>, Self::Error> {
        let path = data_dir()?.join(file_path);
        if !path.exists() {
            return Ok(None); // File does not exist
        }
        let text = std::fs::read_to_string(&path)?;
        toml::from_str(&text)
            .map_err(|e| Error::De(path.clone(), e))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn write<T: serde::Serialize>(
        &mut self,
        file_path: &std::path::Path,
        value: T,
    ) -> Result<(), Self::Error> {
        let path = data_dir()?.join(file_path);
        // TODO: the toml crate's pretty printer actually kind of sucks. I
        // should implement my own and PR it.
        let text = toml::to_string_pretty(&value)
            .map_err(|e| Error::Ser(path.clone(), e))
            .map_err(io::Error::other)?;
        std::fs::write(&path, text)
    }
}

const HELP: &[&str] = &[
    "Controls:",
    "w/a/s/d - move",
    "W/A/S/D - move without turning",
    "b/B - build",
    "q - quit",
];
