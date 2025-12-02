# yt_channel_stats
Youtube channel statistics fetcher terminal tool

You can fetch YouTube channels stats with this Rust based tool.

## Motivations

Some times, I would like to read YT channels stats, especially in programming world!
I can use this tool to find statistics and search by keywords. 


## How to use

This is terminal tool written in RUST, so you can run on almost every modern OS.

You need to login into Google APIs config. page and create your access token to exterl Youtube application.

Next setting up env. variable with secret token: `YT_API_KEY`.

```bash
$ export YT_API_KEY=my_secret_access_token
```

### Commands

```bash
$ cargo run -- -h
yt_channel_stats 0.1.0

USAGE:
    yt_channel_stats <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    channel-meta     Fetch metadata for a channel
    dump-subs        Dump subscriptions (channels liked/subscribed by account)
    help             Prints this message or the help of the given subcommand(s)
    playlists        Fetch public playlists for a channel
    search-videos    Search for videos in a channel by keyword
    stats            Fetch stats for a channel
```

### channel-meta command example results
```

```bash
$ date
thu, 2 dec 2025, 20:45:54 CET

$ cargo run -- channel-meta google $YT_API_KEY
Channel Title: Google
Description: Welcome to Google’s official YouTube channel — join us on our mission to organize the world’s information and make it universally accessible and useful. Subscribe to stay up-to-date on our latest product updates and innovation.

Published At: 2005-09-18T22:37:10Z
Subscribers: 13700000
Total Views: 5285686247
Video Count: 2478
```

### channel-meta command example and results

```bash
$ date
thu, 2 dec 2025, 20:55:54 CET

$ cargo run -- channel-meta google $YT_API_KEY

Channel Title: Google
Description: Welcome to Google’s official YouTube channel — join us on our mission to organize the world’s information and make it universally accessible and useful. Subscribe to stay up-to-date on our latest product updates and innovation.

Published At: 2005-09-18T22:37:10Z
Subscribers: 13700000
Total Views: 5285686247
Video Count: 2478
```

### stats command example and results

```bash
$ cargo run -- stats TEDx $YT_API_KEY views desc > TEDx.views.desc.txt

$ head -n 5 TEDx.views.desc.txt
Title: What is the Internet of Things? And why should you care? | Benson Hougland | TEDxTemecula
Published: 2015-12-02T19:14:53Z
Views: 1100000
Likes: 15000

```

## Known issues with quotas exceed in Google YouTube API

When you will see this example output - this is known issue.
You need to choose:
  1. waiting for next day ;-)
  2. pay for subscription plan for Google!


```bash
$ cargo run -- channel-meta google $YT_API_KEY
Error: reqwest::Error { kind: Decode, source: Error("missing field `items`", line: 13, column: 1) }
```
