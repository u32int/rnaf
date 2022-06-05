use std::error::Error;
use std::io::{Stdout, Write};

use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::ClearType;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute, queue, style::Color, terminal};

use rss::Channel;

// Describes which menu the user is currently in along with the currently selected entry/item.
pub enum TuiState {
    AllFeeds(u16),
    Feed(u16, u16),
    Article(u16, u16),
}

pub struct Tui {
    pub state: TuiState,
}

impl Tui {
    pub fn new() -> Tui {
        Tui {
            state: TuiState::AllFeeds(0),
        }
    }

    pub fn setup(stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        execute!(stdout, terminal::Clear(ClearType::All), cursor::Hide)?;

        Ok(())
    }

    pub fn cleanup(stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
        println!("Cleaning up...");
        disable_raw_mode()?;
        execute!(stdout, cursor::Show)?;

        Ok(())
    }

    pub fn clear(stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
        queue!(stdout, terminal::Clear(ClearType::All))?;
        stdout.flush()?;

        Ok(())
    }

    pub fn draw(&self, stdout: &mut Stdout, feeddata: &Vec<Channel>) -> Result<(), Box<dyn Error>> {
        match self.state {
            TuiState::AllFeeds(sel) => {
                // Draw top bar
                queue!(
                    stdout,
                    cursor::MoveTo(0, 0),
                    PrintStyledContent("rnaf - All Feeds".to_string().with(Color::Blue)),
                )?;

                // Draw feeds
                for (i, feed) in feeddata.iter().enumerate() {
                    if i == sel as usize {
                        queue!(
                            stdout,
                            cursor::MoveTo(4, (i + 1) as u16),
                            PrintStyledContent(
                                format!("{}", feed.title)
                                    .with(Color::Black)
                                    .on(Color::White)
                            ),
                        )?;
                    } else {
                        queue!(
                            stdout,
                            cursor::MoveTo(4, (i + 1) as u16),
                            Print(format!("{}", feed.title)),
                        )?;
                    }
                }
            }
            TuiState::Feed(n, sel) => {
                let currfeed = &feeddata[n as usize];

                // Draw top bar
                queue!(
                    stdout,
                    cursor::MoveTo(0, 0),
                    PrintStyledContent(
                        format!("rnaf - Feed: {}", currfeed.title).with(Color::Blue)
                    ),
                )?;

                // Draw feed items
                for (i, it) in currfeed.items().iter().enumerate() {
                    let title = match &it.title {
                        Some(t) => t,
                        None => "unkwn",
                    };

                    if i == sel as usize {
                        queue!(
                            stdout,
                            cursor::MoveTo(4, (i + 1) as u16),
                            PrintStyledContent(
                                format!("{}", title).with(Color::Black).on(Color::White)
                            ),
                        )?;
                    } else {
                        queue!(
                            stdout,
                            cursor::MoveTo(4, (i + 1) as u16),
                            Print(format!("{}", title)),
                        )?;
                    }
                }
            }
            TuiState::Article(n, i) => {
                let articleitem = feeddata[n as usize].items().iter().nth(i as usize).unwrap();
                // Draw top bar
                let title = articleitem.title().unwrap();
                queue!(
                    stdout,
                    cursor::MoveTo(0, 0),
                    PrintStyledContent(format!("rnaf - Article: {}", title).with(Color::Blue)),
                )?;

                // Draw article info
                let info: Vec<[&str; 2]> = vec![
                    ["title", title],
                    ["author", articleitem.author().unwrap_or("unknown")],
                    ["link", articleitem.link().unwrap_or("unknown")],
                ];
                for (i, meta) in info.iter().enumerate() {
                    queue!(
                        stdout,
                        cursor::MoveTo(2, (i + 1) as u16),
                        Print(format!("{}: {}", meta[0], meta[1])),
                    )?;
                }
            }
        };

        stdout.flush()?;
        Ok(())
    }
}
