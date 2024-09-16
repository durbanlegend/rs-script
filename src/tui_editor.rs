use crokey::{key, KeyCombination};
use crossterm::event::Event::{self, Paste};
use crossterm::event::KeyEvent;
use crossterm::event::MouseEventKind::{
    Down, Drag, Moved, ScrollDown, ScrollLeft, ScrollRight, ScrollUp, Up,
};
use crossterm::terminal;
use mockall::automock;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use scopeguard::ScopeGuard;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::convert::Into;
use std::env::var;
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use tui_textarea::{CursorMove, TextArea};

use crate::colors::{TuiSelectionBg, TUI_SELECTION_BG};
use crate::repl::resolve_term;
use crate::stdin::{apply_highlights, normalize_newlines, show_popup};
use crate::{ThagError, ThagResult};

pub type BackEnd = CrosstermBackend<std::io::StdoutLock<'static>>;
pub type Term = Terminal<BackEnd>;
pub type ResetTermClosure = Box<dyn FnOnce(Term)>;
pub type TermScopeGuard = ScopeGuard<Term, ResetTermClosure>;

#[derive(Default, Serialize, Deserialize)]
pub struct History {
    entries: VecDeque<String>,
    current_index: Option<usize>,
}

/// A trait to allow mocking of the event reader for testing purposes.
#[automock]
pub trait EventReader: Debug {
    /// Read a terminal event.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn read_event(&self) -> ThagResult<Event>;
}

/// A struct to implement real-world use of the event reader, as opposed to use in testing.
#[derive(Debug)]
pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    fn read_event(&self) -> ThagResult<Event> {
        crossterm::event::read().map_err(Into::<ThagError>::into)
    }
}

// Struct to hold data-related parameters
#[allow(dead_code)]
pub struct EditData<'a> {
    pub initial_content: &'a str,
    pub save_path: &'a PathBuf,
    pub history_path: &'a Option<PathBuf>,
    pub history: &'a mut Option<History>,
}

// Struct to hold display-related parameters
pub struct Display<'a> {
    pub title: &'a str,
    pub title_style: Style,
    pub remove_keys: &'a [&'a str],
    pub add_keys: &'a [&'a [&'a str; 2]],
}

#[derive(Debug)]
pub enum KeyAction {
    AbandonChanges,
    Continue, // For other inputs that don't need specific handling
    Quit(bool),
    Save,
    SaveAndExit,
    ShowHelp,
    ToggleHighlight,
    TogglePopup,
}

/// Edit content with a TUI
///
/// # Errors
///
/// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
#[allow(clippy::too_many_lines)]
pub fn edit<R, F>(
    event_reader: &R,
    edit_data: &EditData,
    display: &Display,
    key_handler: F, // closure or function for key handling
) -> ThagResult<KeyAction>
where
    R: EventReader + Debug,
    F: Fn(
        KeyEvent,
        &mut Option<TermScopeGuard>,
        &File,
        &mut TextArea,
        &mut bool,
        &mut bool,
    ) -> ThagResult<KeyAction>,
{
    // Initialize save file
    let save_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(edit_data.save_path)?;

    // Initialize state variables
    let mut popup = false;
    let mut tui_highlight_bg = &*TUI_SELECTION_BG;
    let mut saved = false;

    let mut maybe_term = resolve_term()?;

    // Create the TextArea from initial content
    let mut textarea = TextArea::from(edit_data.initial_content.lines());

    // Set up the display parameters for the textarea
    textarea.set_block(
        Block::default()
            .borders(Borders::NONE)
            .title(display.title)
            .title_style(display.title_style),
    );
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.move_cursor(CursorMove::Bottom);

    // Apply initial highlights
    apply_highlights(&TUI_SELECTION_BG, &mut textarea);

    // Event loop for handling key events
    loop {
        let event = if var("TEST_ENV").is_ok() {
            // Testing or CI
            event_reader.read_event()?
        } else {
            // Real-world interaction
            maybe_term.as_mut().map_or_else(
                || Err("Logic issue unwrapping term we wrapped ourselves".into()),
                |term| {
                    term.draw(|f| {
                        f.render_widget(&textarea, f.area());
                        if popup {
                            show_popup(f, display.remove_keys, display.add_keys);
                        };
                        apply_highlights(tui_highlight_bg, &mut textarea);
                    })
                    .map_err(|e| {
                        eprintln!("Error drawing terminal: {e:?}");
                        e
                    })?;

                    terminal::enable_raw_mode()?;
                    let event = event_reader.read_event();
                    terminal::disable_raw_mode()?;
                    event.map_err(Into::<ThagError>::into)
                },
            )?
        };

        if let Paste(ref data) = event {
            textarea.insert_str(normalize_newlines(data));
        } else {
            match event {
                Event::Key(key_event) => {
                    let key_combination = KeyCombination::from(key_event); // Derive KeyCombination

                    #[allow(clippy::unnested_or_patterns)]
                    match key_combination {
                        key!(ctrl - alt - p) | key!(alt - '<') | key!(alt - up) => {
                            textarea.move_cursor(CursorMove::Top);
                        }
                        key!(ctrl - alt - n) | key!(alt - '>') | key!(alt - down) => {
                            textarea.move_cursor(CursorMove::Bottom);
                        }
                        key!(alt - f) | key!(ctrl - right) => {
                            textarea.move_cursor(CursorMove::WordForward);
                        }
                        key!(alt - b) /* | key!(ctrl - left) */ => {
                            textarea.move_cursor(CursorMove::WordBack);
                        }
                        key!(alt - p) /* | key!(alt - ']') */ | key!(ctrl - up) => {
                            textarea.move_cursor(CursorMove::ParagraphBack);
                        }
                        key!(alt - n) /* | key!(alt - '[') */ | key!(ctrl - down) => {
                            textarea.move_cursor(CursorMove::ParagraphForward);
                        }
                        key!(ctrl - t) => {
                            // Toggle highlighting colours
                            tui_highlight_bg = match tui_highlight_bg {
                                TuiSelectionBg::BlueYellow => &TuiSelectionBg::RedWhite,
                                TuiSelectionBg::RedWhite => &TuiSelectionBg::BlueYellow,
                            };
                            if var("TEST_ENV").is_err() {
                                #[allow(clippy::option_if_let_else)]
                                if let Some(ref mut term) = maybe_term {
                                    term.draw(|_| {
                                        apply_highlights(tui_highlight_bg, &mut textarea);
                                    })?;
                                }
                            }
                        }
                        _ => {
                            // Call the key_handler closure to process events
                            let key_action = key_handler(
                                key_event,
                                &mut maybe_term,
                                &save_file,
                                &mut textarea,
                                &mut popup,
                                &mut saved,
                            )?;

                            match key_action {
                                KeyAction::AbandonChanges | KeyAction::Quit(_) | KeyAction::SaveAndExit => {
                                    break (Ok(key_action))
                                }
                                KeyAction::Continue
                                | KeyAction::Save
                                | KeyAction::ToggleHighlight
                                | KeyAction::TogglePopup => continue,
                                KeyAction::ShowHelp => todo!(),
                            }
                        }
                    }
                }
                Event::Mouse(mouse_event) => {
                    eprintln!("mouse_event={mouse_event:?}");
                    let kind = mouse_event.kind;
                    match kind {
                        Down(x) => eprintln!("Down {x:?}"),
                        Up(x) => eprintln!("Up {x:?}"),
                        Drag(x) => eprintln!("Drag {x:?}"),
                        Moved => eprintln!("Moved"),
                        ScrollDown => eprintln!("ScrollDown"),
                        ScrollUp => eprintln!("ScrollUp"),
                        ScrollLeft => eprintln!("ScrollLeft"),
                        ScrollRight => eprintln!("ScrollRight"),
                    }
                }
                _ => (),
            }
        }
    }
}
