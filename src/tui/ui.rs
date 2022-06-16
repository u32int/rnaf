use std::error::Error;
use std::io::{Stdout, Write};

use crossterm::style::{
    Attribute, Print, PrintStyledContent, SetAttribute, SetForegroundColor, Stylize,
};
use crossterm::terminal::ClearType;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute, queue, style::Color, terminal};

use rss::Channel;

// Describes which menu the user is currently viewing
pub enum TuiState {
    AllFeeds(u16),           // selected feed
    Feed(u16, u16),          // feed number, selected article
    Article(u16, u16, u16),  // feed number, article number, scroll distance
    HelpMenu(Box<TuiState>), // prev state
}

pub struct Tui {
    pub state: TuiState,
    pub termsize: (u16, u16),
}

impl Tui {
    pub fn new() -> Tui {
        Tui {
            state: TuiState::AllFeeds(0),
            termsize: terminal::size().unwrap(),
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
                                feed.title.to_string().with(Color::Black).on(Color::White)
                            ),
                        )?;
                    } else {
                        queue!(
                            stdout,
                            cursor::MoveTo(4, (i + 1) as u16),
                            Print(feed.title.to_string()),
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
                    if i > self.termsize.1 as usize - 2 {
                        break;
                    }

                    let title = match &it.title {
                        Some(t) => t,
                        None => "unkwn",
                    };

                    if i == sel as usize {
                        queue!(
                            stdout,
                            cursor::MoveTo(4, (i + 1) as u16),
                            PrintStyledContent(
                                title.to_string().with(Color::Black).on(Color::White)
                            ),
                        )?;
                    } else {
                        queue!(
                            stdout,
                            cursor::MoveTo(4, (i + 1) as u16),
                            Print(title.to_string()),
                        )?;
                    }
                }
            }
            // n - feed i - article
            TuiState::Article(n, i, scroll) => {
                let currarticle = feeddata[n as usize].items().iter().nth(i as usize).unwrap();
                // Draw top bar
                let title = currarticle.title().unwrap();
                queue!(
                    stdout,
                    cursor::MoveTo(0, 0),
                    PrintStyledContent(format!("rnaf - Article: {}", title).with(Color::Blue)),
                )?;

                // Draw article info
                let info: Vec<[&str; 2]> = vec![
                    ["title", title],
                    ["author", currarticle.author().unwrap_or("unknown")],
                    ["link", currarticle.link().unwrap_or("unknown")],
                ];
                for (j, meta) in info.iter().enumerate() {
                    queue!(
                        stdout,
                        cursor::MoveTo(2, (j + 1) as u16),
                        Print(format!("{}: {}", meta[0], meta[1])),
                    )?;
                }
                // draw description/content

                let desc = currarticle
                    .description()
                    .unwrap()
                    .lines()
                    .collect::<Vec<&str>>();
                let mut cropped = String::new();
                for i in scroll..std::cmp::min(desc.len() as u16, self.termsize.1) {
                    cropped.push_str(desc[i as usize]);
                    cropped.push('\n');
                }

                queue_html_as_string(cropped.to_string(), stdout, self.termsize)?;
            }
            TuiState::HelpMenu(_) => {
                const HELPTEXT: &str = "Keybindings:\n\
					?            - Show this menu\n\
					j/down, k/up - Scroll up and down\n\
					Enter        - Enter/Confirm selection\n\
					q/ESC        - Back/Quit";
                queue!(
                    stdout,
                    cursor::MoveTo(0, 0),
                    PrintStyledContent("rnaf - Help".with(Color::Blue)),
                )?;
                for (i, line) in HELPTEXT.lines().enumerate() {
                    queue!(stdout, cursor::MoveTo(0, i as u16 + 2), Print(line),)?;
                }
            }
        };

        stdout.flush()?;
        Ok(())
    }
}

fn queue_html_as_string(
    html: String,
    stdout: &mut Stdout,
    termsize: (u16, u16),
) -> Result<(), Box<dyn Error>> {
    let html = html.replace('\n', "<RNAFNL>");
    let tags = html.split(&['<', '>']);

    let mut links_buff: Vec<&str> = Vec::new();

    queue!(stdout, cursor::MoveTo(0, 6))?;

    for t in tags {
        // links
        if t.starts_with('a') && t.contains("href=") {
            queue!(
                stdout,
                PrintStyledContent(format!("[{}] ", links_buff.len() + 1).with(Color::Blue)),
                SetAttribute(Attribute::Underlined),
            )?;
            let mut l = t.split("href=");
            let link = l.nth(1).unwrap().split('\"').nth(1).unwrap();
            links_buff.push(link);
        } else {
            // the rest
            match t {
                "RNAFNL" => {
                    queue!(
                        stdout,
                        cursor::MoveDown(1),
                        cursor::MoveTo(0, cursor::position()?.1),
                    )?;
                }
                "code" => queue!(stdout, SetForegroundColor(Color::Yellow))?,
                "/code" => queue!(stdout, SetForegroundColor(Color::Reset))?,
                "pre" => queue!(stdout, cursor::MoveTo(0, cursor::position()?.1))?,
                "/pre" => {}
                "strong" | "b" => queue!(stdout, SetAttribute(Attribute::Bold))?,
                "/strong" | "/b" => queue!(stdout, SetAttribute(Attribute::NoBold))?,
                "ul" | "/ul" => {}
                "li" => {
                    queue!(
                        stdout,
                        cursor::MoveDown(1),
                        cursor::MoveTo(0, cursor::position()?.1),
                        Print("- "),
                    )?;
                }
                "p" => {
                    queue!(
                        stdout,
                        cursor::MoveDown(1),
                        cursor::MoveTo(0, cursor::position()?.1),
                    )?;
                }
                "/li" => {}
                "/p" => {}
                "/a" => queue!(stdout, SetAttribute(Attribute::NoUnderline))?,
                _ => {
                    if t.len() == 0 {
                        continue;
                    }

                    let mut start = 0;
                    let end = t.len();
                    let screensize = termsize.0 as usize - 10;
                    while end - start > screensize {
                        queue!(stdout, Print(&t[start..start + screensize]),)?;
                        start += screensize;
                    }
                    queue!(stdout, Print(&t[start..end]),)?;
                }
            }
        }
    }
    queue!(stdout, SetAttribute(Attribute::Reset))?;

    Ok(())
}
