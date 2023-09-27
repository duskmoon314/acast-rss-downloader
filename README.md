# acast-rss-downloader

Download all mp3 files of a podcast from an acast RSS feed

## Usage

```shell
acast-rss-downloader <URL>

# cargo build and run
cargo run -- <URL>
```

All mp3 files will be downloaded to `./<podcast-name>/` directory with name `<acast:episodeUrl>.mp3`.

Currenlty, only `<itunes:image>` is used to set the cover image of the mp3 file.
