use std::io::Write;

use clap::Parser;
use id3::{frame::Picture, Content, Frame, TagLike, Version};
use rss::Channel;
use url::Url;

#[derive(Debug, Parser)]
struct Args {
    /// RSS feed URL
    url: Url,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arg = Args::parse();
    println!("Downloading audio from {}", arg.url);

    // Get the feed from the url
    let feed = reqwest::get(arg.url).await?.bytes().await?;

    // Use rss crate to parse the channel
    let feed = Channel::read_from(&feed[..])?;

    // Use the title as the folder name for the podcast
    let podcast_title = feed.title().to_owned();
    std::fs::create_dir_all(&podcast_title)?;

    let handles = feed
        .items()
        .iter()
        .map(|item| tokio::spawn(download(item.clone(), podcast_title.clone())));

    futures::future::join_all(handles).await;

    Ok(())
}

async fn download(item: rss::Item, dir: String) -> anyhow::Result<()> {
    let title = item
        .extensions()
        .get("acast")
        .unwrap()
        .get("episodeUrl")
        .unwrap()
        .first()
        .unwrap()
        .value()
        .unwrap()
        .to_owned();
    let url = item.enclosure().unwrap().url().parse::<Url>().unwrap();
    let image_url = item
        .itunes_ext()
        .unwrap()
        .image()
        .unwrap()
        .parse::<Url>()
        .unwrap();

    // Get the audio file and save it to <dir>/<title>.mp3
    let audio = reqwest::get(url).await?.bytes().await?;
    let mut file = std::fs::File::create(format!("{dir}/{title}.mp3"))?;
    file.write_all(&audio)?;

    // Get the image file and write it to ID3 tag
    let image = reqwest::get(image_url).await?.bytes().await?;
    let mut tag = id3::Tag::read_from_path(format!("{dir}/{title}.mp3"))?;
    let picture = Picture {
        mime_type: "image/jpeg".to_owned(),
        picture_type: id3::frame::PictureType::CoverFront,
        description: String::new(),
        data: image.to_vec(),
    };
    tag.add_frame(Frame::with_content("APIC", Content::Picture(picture)));
    tag.write_to_path(format!("{dir}/{title}.mp3"), Version::Id3v24)?;

    Ok(())
}
