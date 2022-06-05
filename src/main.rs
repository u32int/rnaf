use std::error::Error;
use std::time::Duration;

use crossterm::event::{poll, read, Event};

mod app;
mod rssdata;
mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = app::App::new();
    let mut stdout = std::io::stdout();

    let projectroot = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let paths = vec![
        format!("{}/examplefeeds/arch.rss", projectroot),
        format!("{}/examplefeeds/debian.rss", projectroot),
    ];
    let data = rssdata::util::get_all(paths).await;
    // This is not ideal but async traits are worse
    app.data.feeds = data;

    app.init(&mut stdout)?;

    while app.run {
        if poll(Duration::from_millis(1_000))? {
            let ev = read()?;

            match ev {
                Event::Key(ev) => tui::keyhandler::handle_keyevent(ev, &mut app, &mut stdout)?,
                Event::Mouse(_ev) => unimplemented!(),
                Event::Resize(_w, _h) => unimplemented!(),
            }

            app.tui.draw(&mut stdout, &app.data.feeds)?;
        }
    }

    app.cleanup(&mut stdout)?;
    Ok(())
}
