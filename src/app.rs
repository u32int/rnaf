use crate::tui::ui::Tui;
use std::error::Error;
use std::io::Stdout;

use rss::Channel;

pub struct RssData {
    pub feeds: Vec<Channel>,
}

impl RssData {
    fn new() -> RssData {
        RssData { feeds: Vec::new() }
    }
}

pub struct App {
    pub run: bool,
    pub tui: Tui,
    pub data: RssData,
}

impl App {
    pub fn new() -> App {
        App {
            run: true,
            tui: Tui::new(),
            data: RssData::new(),
        }
    }

    pub fn init(&self, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
        let minsize = (60, 30);
        if self.tui.termsize.0 < minsize.0 || self.tui.termsize.1 < minsize.1 {
            eprintln!(
                "Error! Terminal size too small!\nRequired size: {}x{}\nCurrent size:{}x{}",
                minsize.0, minsize.1, self.tui.termsize.0, self.tui.termsize.1
            );
            std::process::exit(1);
        }

        Tui::setup(stdout)?;

        self.tui.draw(stdout, &self.data.feeds)?;
        Ok(())
    }

    pub fn cleanup(&self, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
        Tui::cleanup(stdout)?;
        Ok(())
    }
}
