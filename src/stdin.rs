#![allow(clippy::uninlined_format_args)]
use crate::errors::BuildRunError;
use crate::log;
use crate::logging::Verbosity;

use crossterm::event::{
    DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event::{self, Paste},
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use lazy_static::lazy_static;
use mockall::{automock, predicate::str};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin};
use ratatui::prelude::Rect;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;
use regex::Regex;
use std::error::Error;
use std::io::{self, BufRead, IsTerminal};
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[automock]
pub trait EventReader {
    /// Read a `crossterm` event.
    /// # Errors
    ///
    /// If the timeout expires then an error is returned and buf is unchanged.
    fn read_event(&self) -> Result<Event, std::io::Error>;
}

pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    fn read_event(&self) -> Result<Event, std::io::Error> {
        crossterm::event::read()
    }
}

#[allow(dead_code)]
fn main() -> Result<(), Box<dyn Error>> {
    let event_reader = CrosstermEventReader;
    for line in &edit(&event_reader)? {
        log!(Verbosity::Normal, "{line}");
    }
    Ok(())
}

/// Edit the stdin stream.
///
///
/// # Examples
///
/// ```
/// use rs_script::stdin::edit;
///
/// let buf = vec!["Hello", "world"];
/// assert_eq!(edit(event_reader), Ok(buf));
/// ```
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
/// # Panics
///
/// If the terminal cannot be reset.
pub fn edit<R: EventReader>(event_reader: &R) -> Result<Vec<String>, Box<dyn Error>> {
    let input = std::io::stdin();

    let initial_content = if input.is_terminal() {
        // No input available
        String::new()
    } else {
        read()?
    };

    let mut popup = false;
    let mut alt_highlights = false;

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode().map_err(|e| {
        println!("Error enabling raw mode: {:?}", e);
        e
    })?;
    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )
    .map_err(|e| {
        println!("Error executing terminal commands: {:?}", e);
        e
    })?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).map_err(|e| {
        println!("Error creating terminal: {:?}", e);
        e
    })?;
    // Ensure terminal will get reset when it goes out of scope.
    let mut term = scopeguard::guard(terminal, |term| {
        reset_term(term).expect("Error resetting terminal");
    });

    let mut textarea = TextArea::from(initial_content.lines());

    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title("Enter / paste / edit Rust script. Ctrl+D: submit  Ctrl+Q: quit  Ctrl+L: keys")
            .title_style(Style::default().fg(Color::Yellow).bold().italic()),
    );
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.set_selection_style(Style::default().bg(Color::Blue));
    textarea.set_cursor_style(Style::default().on_magenta());
    textarea.set_cursor_line_style(Style::default().on_dark_gray());

    textarea.move_cursor(CursorMove::Bottom);

    apply_highlights(alt_highlights, &mut textarea);

    loop {
        term.draw(|f| {
            f.render_widget(&textarea, f.size());
            if popup {
                show_popup(f);
            }
            apply_highlights(alt_highlights, &mut textarea);
        })
        .map_err(|e| {
            println!("Error drawing terminal: {:?}", e);
            e
        })?;
        let event = event_reader.read_event().map_err(|e| {
            println!("Error reading event: {:?}", e);
            e
        })?;
        if let Paste(data) = event {
            textarea.insert_str(normalize_newlines(&data));
        } else {
            let input = Input::from(event.clone());
            match input {
                Input {
                    key: Key::Char('q'),
                    ctrl: true,
                    ..
                } => {
                    return Err(Box::new(BuildRunError::Cancelled));
                }
                Input {
                    key: Key::Char('d'),
                    ctrl: true,
                    ..
                } => break,
                Input {
                    key: Key::Char('l'),
                    ctrl: true,
                    ..
                } => popup = !popup,
                Input {
                    key: Key::Char('t'),
                    ctrl: true,
                    ..
                } => {
                    alt_highlights = !alt_highlights;
                    term.draw(|_| {
                        apply_highlights(alt_highlights, &mut textarea);
                    })?;
                }

                input => {
                    textarea.input(input);
                }
            }
        }
    }

    Ok(textarea.lines().to_vec())
}

/// Prompt for and read Rust source code from stdin.
///
/// # Examples
///
/// ```
/// use rs_script::stdin::read;
///
/// assert_eq!(read(), Ok("Hello world!"));
/// ```
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read() -> Result<String, std::io::Error> {
    log!(Verbosity::Normal, "Enter or paste lines of Rust source code at the prompt and press Ctrl-D on a new line when done");
    let input = &mut std::io::stdin().lock();
    let buffer = read_to_string(input)?;
    Ok(buffer)
}

///
///
/// # Examples
///
/// ```
/// use rs_script::stdin::read_to_string;
///
/// let stdin = std::io::stdin();
/// let mut input = stdin.lock();
/// assert_eq!(read_to_string(&mut input), );
/// assert_eq!(input, );
/// ```
///
/// # Errors
///
/// If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
pub fn read_to_string<R: BufRead>(input: &mut R) -> Result<String, io::Error> {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[must_use]
pub fn normalize_newlines(input: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\r\n?").unwrap();
    }
    RE.replace_all(input, "\n").to_string()
}

pub fn apply_highlights(alt_highlights: bool, textarea: &mut TextArea) {
    if alt_highlights {
        // Dark theme-friendly colors
        textarea.set_selection_style(Style::default().bg(Color::Cyan).fg(Color::Black));
        textarea.set_cursor_style(Style::default().bg(Color::LightYellow).fg(Color::Black));
        textarea.set_cursor_line_style(Style::default().bg(Color::DarkGray).fg(Color::White));
    } else {
        // Light theme-friendly colors
        textarea.set_selection_style(Style::default().bg(Color::Blue).fg(Color::White));
        textarea.set_cursor_style(Style::default().bg(Color::LightRed).fg(Color::White));
        textarea.set_cursor_line_style(Style::default().bg(Color::Gray).fg(Color::Black));
    }
}

// fn insert_line(textarea: &mut TextArea, line: &str) {
//     textarea.insert_str(line);
//     if cfg!(windows) {
//         textarea.insert_str("\r");
//     }
//     textarea.insert_newline();
// }

fn reset_term(
    mut term: Terminal<CrosstermBackend<io::StdoutLock<'_>>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
fn show_popup(f: &mut ratatui::prelude::Frame) {
    let area = centered_rect(90, NUM_ROWS as u16 + 5, f.size());
    let inner = area.inner(Margin {
        vertical: 2,
        horizontal: 2,
    });
    let block = Block::default()
        .borders(Borders::ALL)
        .title(
            Title::from("Platform-dependent key mappings (YMMV)")
                .alignment(ratatui::layout::Alignment::Center),
        )
        .title(Title::from("(Ctrl+L to toggle)").alignment(Alignment::Center))
        .add_modifier(Modifier::BOLD);
    f.render_widget(Clear, area);
    //this clears out the background
    f.render_widget(block, area);
    let row_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints::<Vec<Constraint>>(
            std::iter::repeat(Constraint::Ratio(1, NUM_ROWS as u32))
                .take(NUM_ROWS)
                .collect::<Vec<Constraint>>(),
        );
    let rows = row_layout.split(inner);

    for (i, row) in rows.iter().enumerate() {
        let col_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(45), Constraint::Length(43)].as_ref());
        let cells = col_layout.split(*row);
        for n in 0..=1 {
            let mut widget = Paragraph::new(MAPPINGS[i][n]);
            if i == 0 {
                widget = widget.add_modifier(Modifier::BOLD);
            } else {
                widget = widget.remove_modifier(Modifier::BOLD);
            }
            f.render_widget(widget, cells[n]);
        }
    }
}

fn centered_rect(max_width: u16, max_height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Max(max_height),
        Constraint::Fill(1),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Max(max_width),
        Constraint::Fill(1),
    ])
    .split(popup_layout[1])[1]
}

const MAPPINGS: &[[&str; 2]; 33] = &[
    ["Key Bindings", "Description"],
    ["Shift+arrow keys", "Select/deselect ← chars→  / ↑ lines↓"],
    [
        "Shift+Ctrl+arrow keys",
        "Select/deselect ← words→  / ↑ paras↓",
    ],
    ["Ctrl+D", "Submit"],
    ["Ctrl+Q", "Cancel and quit"],
    ["Ctrl+H, Backspace", "Delete character before cursor"],
    ["Ctrl+I, Tab", "Indent"],
    ["Ctrl+M, Enter", "Insert newline"],
    ["Ctrl+K", "Delete from cursor to end of line"],
    ["Ctrl+J", "Delete from cursor to start of line"],
    ["Ctrl+W, Alt+<, Backspace", "Delete one word before cursor"],
    ["Alt+D, Delete", "Delete one word from cursor position"],
    ["Ctrl+U", "Undo"],
    ["Ctrl+R", "Redo"],
    ["Ctrl+C", "Copy (yank) selected text"],
    ["Ctrl+X", "Cut (yank) selected text"],
    ["Ctrl+Y", "Paste yanked text"],
    ["Ctrl+V, Shift+Ins, Cmd+V", "Paste from system clipboard"],
    ["Ctrl+F, →", "Move cursor forward one character"],
    ["Ctrl+B, ←", "Move cursor backward one character"],
    ["Ctrl+P, ↑", "Move cursor up one line"],
    ["Ctrl+N, ↓", "Move cursor down one line"],
    ["Alt+F, Ctrl+→", "Move cursor forward one word"],
    ["Atl+B, Ctrl+←", "Move cursor backward one word"],
    ["Alt+] or P, Ctrl+↑", "Move cursor up one paragraph"],
    ["Alt+[ or N, Ctrl+↓", "Move cursor down one paragraph"],
    [
        "Ctrl+E, End, Ctrl+Alt+F or → , Cmd+→",
        "Move cursor to end of line",
    ],
    [
        "Ctrl+A, Home, Ctrl+Alt+B or ← , Cmd+←",
        "Move cursor to start of line",
    ],
    ["Alt+<, Ctrl+Alt+P or ↑", "Move cursor to top of file"],
    ["Alt+>, Ctrl+Alt+N or↓", "Move cursor to bottom of file"],
    ["PageDown, Cmd+↓", "Page down"],
    ["Alt+V, PageUp, Cmd+↑", "Page up"],
    ["Ctrl+T", "Toggle highlight colours"],
];
const NUM_ROWS: usize = MAPPINGS.len();
