use crate::{Dir, Input, IsShift, Menu, Platform, State, Tile};
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::style::{self, Attribute, Color, Colors, Print};
use crossterm::terminal;
use crossterm::{execute, queue};
use std::io::{self, Write, stdout};
use std::path::{Path, PathBuf};

/*
fn line_ending() -> &'static str {
    // We need this line ending, because in raw mode, in some terminals, `\n`
    // does not return to the start of the line.
    // And, notably, `writeln!` does not print a `\r`.
    "\r\n"
}
*/

mod border {
    use std::io::{self, Write};

    pub const TL: char = '┏';
    pub const T: char = '━';
    pub const TR: char = '┓';
    pub const L: char = '┃';
    pub const R: char = '┃';
    pub const BL: char = '┗';
    pub const B: char = '━';
    pub const BR: char = '┛';

    pub fn bottom_row(output: &mut impl Write, inner_width: u16) -> io::Result<()> {
        write!(output, "{}", BL)?;
        for _ in 0..inner_width {
            write!(output, "{}", B)?;
        }
        write!(output, "{}", BR)
    }

    pub fn top_row(output: &mut impl Write, inner_width: u16) -> io::Result<()> {
        write!(output, "{}", TL)?;
        for _ in 0..inner_width {
            write!(output, "{}", T)?;
        }
        write!(output, "{}", TR)
    }
}

fn data_dir() -> io::Result<PathBuf> {
    // TODO: Maybe return a result?
    let dir = dirs::data_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find data directory"))?;
    let out = dir.join("j-minecraft-2d");
    if !out.exists() {
        std::fs::create_dir_all(&out)?;
    }
    Ok(out)
}

fn on_letter_pressed(char: char) -> Option<Input> {
    match char {
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
        'i' | 'I' => Some(Input::OpenInventory),
        _ => None,
    }
}

fn on_key_event(key_event: KeyEvent) -> Option<Input> {
    // We want to skip release events because they are not the pressing of a button.
    if key_event.kind == event::KeyEventKind::Release {
        return None;
    }
    match key_event.code {
        KeyCode::Char(ch) => on_letter_pressed(ch),
        KeyCode::Esc => Some(Input::CloseMenu),
        _ => None,
        /* Other types of key-event codes:
         * `KeyCode::Backspace`
         * `KeyCode::Enter`
         * `KeyCode::Left`
         * `KeyCode::Right`
         * `KeyCode::Up`
         * `KeyCode::Down`
         * `KeyCode::Home`
         * `KeyCode::End`
         * `KeyCode::PageUp`
         * `KeyCode::PageDown`
         * `KeyCode::Tab`
         * `KeyCode::BackTab`
         * `KeyCode::Delete`
         * `KeyCode::Insert`
         * `KeyCode::F(_)`
         * `KeyCode::Null`
         * `KeyCode::CapsLock`
         * `KeyCode::ScrollLock`
         * `KeyCode::NumLock`
         * `KeyCode::PrintScreen`
         * `KeyCode::Pause`
         * `KeyCode::Menu`
         * `KeyCode::KeypadBegin`
         * `KeyCode::Media(media_key_code)`
         * `KeyCode::Modifier(modifier_key_code)`
         */
    }
}

fn get_input() -> Option<Input> {
    // TODO: Currently, this buffers input. So if you spam a key, it will keep
    // being registered as pressed even after you let go of the button (if there
    // is some lag). To avoid this, we want another thread reading input and
    // blocking, and sending them individually, but to a 1-length buffer.
    let event = crossterm::event::read().expect("Failed to read input");
    match event {
        Event::Key(key_event) => on_key_event(key_event),
        _ => None,
        /* Other types of events:
         *
         * `Event::FocusGained`
         * `Event::FocusLost`
         * `Event::Mouse(mouse_event)`
         * `Event::Paste(_)`
         * `Event::Resize(_, _)`
         */
    }
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

    pub const fn single(c: char) -> Self {
        Self::new(c, c)
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

impl From<char> for Chars {
    fn from(char: char) -> Self {
        Self::single(char)
    }
}

impl From<[char; 2]> for Chars {
    fn from(chars: [char; 2]) -> Self {
        Self::new(chars[0], chars[1])
    }
}

const SHADES: [char; 4] = ['░', '▒', '▓', '█'];

/// A tile get's drawn to two characters because most fonts are taller than
/// they are wide.
fn draw_tile(tile: Tile) -> Chars {
    match tile {
        Tile::WallFull => ['█', '█'].into(),
        Tile::WallHalf => ['▓', '▓'].into(),
        Tile::WallLow => ['▒', '▒'].into(),
        Tile::Empty => [' ', ' '].into(),
        Tile::Wood(n) => Chars::single(SHADES[n.min(3) as usize]).with_fg(Color::DarkYellow),
    }
}

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

fn draw(state: &State, output: &mut impl io::Write, width: u16, height: u16) -> io::Result<()> {
    let outer_width = width & !1 /* Ensure even */;
    // let outer_height = height - 2 /* For living space for text below */;
    let outer_height = height;
    let inner_width = outer_width - 2 /* For the frame */;
    let inner_height = outer_height - 2 /* For the frame */;
    let rows = inner_height;
    let cells_in_a_row = inner_width / 2;

    queue!(output, style::ResetColor)?;

    queue!(output, cursor::MoveTo(0, 0))?;
    border::top_row(output, inner_width)?;

    for row in 0..rows {
        queue!(output, cursor::MoveTo(0, row + 1))?;
        write!(output, "{}", border::L)?;
        for col in 0..cells_in_a_row {
            let pos = (
                state.player_pos.0 + col as i32 - cells_in_a_row as i32 / 2,
                state.player_pos.1 + row as i32 - rows as i32 / 2,
            );
            // TODO: this should just check against row and col, not the pos.
            let chars = if pos == state.player_pos {
                queue!(output, cursor::SavePosition,)?;
                player(state.player_dir)
            } else {
                let tile = state.get_tile(pos);
                draw_tile(tile)
            };
            chars.write(output)?;
        }
        write!(output, "{}", border::R)?;
    }

    queue!(output, cursor::MoveTo(0, rows + 1))?;
    border::bottom_row(output, inner_width)?;

    queue!(output, cursor::MoveTo(0, rows + 1))?;
    write!(output, "XY: {} {}", state.player_pos.0, state.player_pos.1,)?;

    queue!(
        output,
        style::ResetColor,
        cursor::RestorePosition,
        cursor::MoveDown(2),
        cursor::MoveLeft((state.message.len() / 2) as u16),
        Print(&state.message),
    )?;

    match state.menu {
        Menu::None => (),
        Menu::Inventory => draw_inventory(
            state,
            output,
            (width / 4, height / 4),
            (width / 2, height / 2),
        )?,
    }

    Ok(())
}

fn draw_inventory(
    state: &State,
    output: &mut impl io::Write,
    (left, top): (u16, u16),
    (width, height): (u16, u16),
) -> io::Result<()> {
    let bottom = top + height - 1;
    let inner_width = width - 2;

    queue!(output, cursor::MoveTo(left, top))?;
    border::top_row(output, inner_width)?;

    // Clear the inside
    for row in top + 1..bottom {
        queue!(
            output,
            cursor::MoveTo(left, row),
            Print(border::L),
            Print(" ".repeat(inner_width as usize)),
            Print(border::R),
        )?;
    }

    let draw_player_at = (left + 3, top + 2);
    queue!(output, cursor::MoveTo(draw_player_at.0, draw_player_at.1))?;
    player(state.player_dir).write(output)?;
    queue!(output, style::ResetColor)?;

    queue!(output, cursor::MoveTo(left + 1, top + 4))?;
    write!(output, "{}", "-".repeat(inner_width as usize))?;

    for (i, (item, count)) in state.inventory.iter().enumerate() {
        queue!(output, cursor::MoveTo(left + 6, top + 6 + i as u16))?;
        let name = item.name();
        let is_selected = Some(&item) == state.selected_item.as_ref();
        let selected: Colors = Colors::new(Color::Black, Color::White);
        if is_selected {
            queue!(
                output,
                style::SetColors(selected),
                // style::SetAttribute(Attribute::Underlined),
            )?;
        }
        let prefix = if is_selected { '>' } else { ' ' };
        if count == 1 {
            write!(output, "{prefix} {name}")?;
        } else {
            write!(output, "{prefix} {name} ✗ {count}")?;
        }
        if is_selected {
            queue!(
                output,
                style::ResetColor,
                style::SetAttribute(Attribute::Reset)
            )?;
        }
    }

    queue!(output, cursor::MoveTo(left, bottom))?;
    border::bottom_row(output, inner_width)?;

    Ok(())
}

/// TODO: Rename
#[derive(Debug)]
enum Error {
    Ser(PathBuf, toml::ser::Error),
    De(PathBuf, toml::de::Error),
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerminalPlatform;

impl TerminalPlatform {
    pub const fn new() -> Self {
        TerminalPlatform
    }

    fn read<T: serde::de::DeserializeOwned>(
        &mut self,
        file_path: &Path,
    ) -> Result<Option<T>, io::Error> {
        let path = data_dir()?.join(file_path);
        if !path.exists() {
            return Ok(None); // File does not exist
        }
        let text = std::fs::read_to_string(&path)?;
        toml::from_str(&text)
            .map_err(|e| Error::De(path.clone(), e))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn write<T: serde::Serialize>(&mut self, file_path: &Path, value: T) -> Result<(), io::Error> {
        let path = data_dir()?.join(file_path);
        // TODO: the toml crate's pretty printer actually kind of sucks. I
        // should implement my own and PR it.
        let text = toml::to_string_pretty(&value)
            .map_err(|e| Error::Ser(path.clone(), e))
            .map_err(io::Error::other)?;
        std::fs::write(&path, text)
    }
}

impl Platform for TerminalPlatform {
    type Error = io::Error;

    fn init(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        #[cfg(unix)]
        queue!(
            stdout(),
            event::PushKeyboardEnhancementFlags(event::KeyboardEnhancementFlags::empty()),
        );
        execute!(stdout(), terminal::EnterAlternateScreen,)?;
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        #[cfg(unix)]
        queue!(stdout(), event::PopKeyboardEnhancementFlags,);
        execute!(stdout(), terminal::LeaveAlternateScreen,)?;
        Ok(())
    }

    fn ask_for_input(&mut self) -> io::Result<Option<Input>> {
        Ok(get_input())
    }

    fn draw(&mut self, state: &State) -> io::Result<()> {
        queue!(
            stdout(),
            // terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )?;
        let mut out = vec![];
        let (w, h) = terminal::size()?;
        let (w, h) = (w as _, h as _);
        draw(state, &mut out, w, h)?;
        io::stdout().write_all(&out)?;
        execute!(
            stdout(),
            style::ResetColor,
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
            cursor::MoveTo(1, 6),
            Print(HELP[5]),
            cursor::MoveTo(1, 7),
            Print(HELP[6]),
        )?;
        Ok(())
    }

    fn save(&mut self, state: &State) -> io::Result<()> {
        // TODO: Make a backup.
        self.write(Path::new("save"), state)
    }

    fn load(&mut self) -> io::Result<Option<State>> {
        self.read(Path::new("save"))
    }
}

const HELP: &[&str] = &[
    "Controls:",
    "w/a/s/d - move",
    "W/A/S/D - move without turning",
    "b/B - build",
    "i/I - open inventory",
    "Esc - close menu",
    "q - quit",
];
