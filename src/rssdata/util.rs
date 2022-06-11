use rss::Channel;
use std::error::Error;
use std::io::BufReader;

use futures::future::join_all;

async fn get_feed(url: &str) -> Result<Channel, Box<dyn Error>> {
    let feed = reqwest::get(url).await?.bytes().await?;
    let ch = Channel::read_from(&feed[..])?;

    Ok(ch)
}

async fn feed_from_file(path: &str) -> Result<Channel, Box<dyn Error>> {
    eprintln!("getting path: {}", path);
    let f = tokio::fs::File::open(path).await?.into_std().await;
    let ch = Channel::read_from(BufReader::new(f))?;

    Ok(ch)
}

pub async fn get_all(feeds: Vec<String>) -> Vec<Channel> {
    let ftrs = feeds.iter().map(|p| feed_from_file(p));
    let result = join_all(ftrs).await.into_iter().flatten().collect();

    result
}
