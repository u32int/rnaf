use crossterm::event::{KeyCode, KeyEvent};
use std::{error::Error, io::Stdout};

use super::ui::{Tui, TuiState};
use crate::app::App;

pub fn handle_keyevent(
    ev: KeyEvent,
    app: &mut App,
    stdout: &mut Stdout,
) -> Result<(), Box<dyn Error>> {
    match &app.tui.state {
        TuiState::AllFeeds(sel) => match ev.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.run = false;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.tui.state =
                    TuiState::AllFeeds((sel + 1).clamp(0, app.data.feeds.len() as u16 - 1));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.tui.state = TuiState::AllFeeds(
                    (sel.checked_sub(1).unwrap_or(0)).clamp(0, app.data.feeds.len() as u16),
                );
            }
            KeyCode::Enter => {
                Tui::clear(stdout)?;
                app.tui.state = TuiState::Feed(*sel, 0);
            }
            _ => {}
        },

        TuiState::Feed(n, sel) => match ev.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                Tui::clear(stdout)?;
                app.tui.state = TuiState::AllFeeds(0);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                use std::cmp::min;
                app.tui.state = TuiState::Feed(
                    *n,
                    (sel + 1).clamp(
                        0,
                        min(
                            app.data.feeds[*n as usize].items.len() as u16 - 1,
                            app.tui.termsize.1 - 2,
                        ),
                    ),
                );
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.tui.state = TuiState::Feed(
                    *n,
                    (sel.checked_sub(1).unwrap_or(0))
                        .clamp(0, app.data.feeds[*n as usize].items.len() as u16),
                );
            }
            KeyCode::Enter => {
                Tui::clear(stdout)?;
                app.tui.state = TuiState::Article(*n, *sel, 0);
            }

            _ => {}
        },

        TuiState::Article(n, i, s) => match ev.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                Tui::clear(stdout)?;
                app.tui.state = TuiState::Feed(*n, *i);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                Tui::clear(stdout)?;
                app.tui.state = TuiState::Article(*n, *i, *s + 1)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                Tui::clear(stdout)?;
                app.tui.state = TuiState::Article(*n, *i, s.checked_sub(1).unwrap_or(0));
            }
            KeyCode::Char('?') => {
                Tui::clear(stdout)?;
                app.tui.state = TuiState::HelpMenu(Box::new(TuiState::Article(*n, *i, *s)));
            }
            _ => {}
        },

        TuiState::HelpMenu(prevstate) => match ev.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                Tui::clear(stdout)?;

                app.tui.state = match **prevstate {
                    TuiState::AllFeeds(n) => TuiState::AllFeeds(n),
                    TuiState::Feed(n, sel) => TuiState::Feed(n, sel),
                    TuiState::Article(n, i, s) => TuiState::Article(n, i, s),
                    _ => TuiState::AllFeeds(0),
                }
            }
            _ => {}
        },
    }

    Ok(())
}
