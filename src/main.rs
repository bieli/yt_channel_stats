use reqwest::Client;
use serde::Deserialize;
use structopt::StructOpt;

#[derive(StructOpt)]
enum Command {
    Stats {
        channel_handle: String,
        api_key: String,
        sort_key: Option<String>,
        sort_order: Option<String>,
    },
    DumpSubs {
        oauth_token: String,
    },
    ChannelMeta {
        channel_handle: String,
        api_key: String,
    },
    Playlists {
        channel_handle: String,
        api_key: String,
    },
    SearchVideos {
        channel_handle: String,
        api_key: String,
        query: String,
    },
}

#[derive(Deserialize)]
struct SearchResponse {
    items: Vec<SearchItem>,
}
#[derive(Deserialize)]
struct SearchItem {
    id: SearchId,
}
#[derive(Deserialize)]
struct SearchId {
    #[serde(rename = "channelId")]
    channel_id: String,
}

#[derive(Deserialize)]
struct ChannelResponse {
    items: Vec<ChannelItem>,
}

#[derive(Deserialize)]
struct ChannelItem {
    #[serde(default)]
    snippet: Option<ChannelSnippet>,
    #[serde(default)]
    statistics: Option<ChannelStatistics>,
    #[serde(rename = "contentDetails")]
    content_details: Option<ChannelContentDetails>,
}


#[derive(Deserialize)]
struct ChannelSnippet {
    title: String,
    description: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
}
#[derive(Deserialize)]
struct ChannelStatistics {
    #[serde(rename = "subscriberCount")]
    subscriber_count: String,
    #[serde(rename = "viewCount")]
    view_count: String,
    #[serde(rename = "videoCount")]
    video_count: String,
}
#[derive(Deserialize)]
struct ChannelContentDetails {
    #[serde(rename = "relatedPlaylists")]
    related_playlists: RelatedPlaylists,
}
#[derive(Deserialize)]
struct RelatedPlaylists {
    uploads: String,
}

#[derive(Deserialize)]
struct PlaylistResponse {
    items: Vec<PlaylistItem>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}
#[derive(Deserialize)]
struct PlaylistItem {
    #[serde(rename = "contentDetails")]
    content_details: ContentDetails,
}
#[derive(Deserialize)]
struct ContentDetails {
    #[serde(rename = "videoId")]
    video_id: String,
}

#[derive(Deserialize)]
struct VideoResponse {
    items: Vec<VideoItem>,
}
#[derive(Deserialize)]
struct VideoItem {
    snippet: Snippet,
    statistics: Statistics,
}

#[derive(Deserialize)]
struct Snippet {
    title: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
}
#[derive(Deserialize)]
struct Statistics {
    #[serde(rename = "viewCount")]
    view_count: String,
    #[serde(rename = "likeCount")]
    like_count: Option<String>,
}

#[derive(Deserialize)]
struct PlaylistListResponse {
    items: Vec<PlaylistListItem>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}
#[derive(Deserialize)]
struct PlaylistListItem {
    snippet: PlaylistSnippet,
}
#[derive(Deserialize)]
struct PlaylistSnippet {
    title: String,
    description: String,
}

struct VideoData {
    title: String,
    published: String,
    views: u64,
    likes: u64,
}

#[derive(Deserialize)]
struct VideoSearchResponse {
    #[serde(default)]
    items: Vec<VideoSearchItem>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Deserialize)]
struct VideoSearchItem {
    id: VideoSearchId,
    snippet: VideoSearchSnippet,
}

#[derive(Deserialize)]
struct VideoSearchId {
    #[serde(rename = "videoId")]
    video_id: String,
}

#[derive(Deserialize)]
struct VideoSearchSnippet {
    title: String,
    description: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
}


async fn run_stats(
    channel_handle: String,
    api_key: String,
    sort_key: Option<String>,
    sort_order: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Resolve handle -> channelId
    let query = if channel_handle.starts_with('@') {
        channel_handle.clone()
    } else {
        format!("@{}", channel_handle)
    };
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=channel&q={}&key={}",
        query, api_key
    );
    //println!("Search URL: {}", url);
    let search_text = client.get(&url).send().await?.text().await?;
    //println!("Search raw JSON: {}", search_text);
    let search_resp: SearchResponse = serde_json::from_str(&search_text)?;
    if search_resp.items.is_empty() {
        eprintln!("No channel found for handle {}", channel_handle);
        return Ok(());
    }
    let channel_id = &search_resp.items[0].id.channel_id;
    //println!("Resolved channel_id: {}", channel_id);

    // Get uploads playlist
    let url = format!(
        "https://www.googleapis.com/youtube/v3/channels?part=contentDetails&id={}&key={}",
        channel_id, api_key
    );
    //println!("Channel URL: {}", url);
    let channel_text = client.get(&url).send().await?.text().await?;
    //println!("Channel raw JSON: {}", channel_text);
    let channel_resp: ChannelResponse = serde_json::from_str(&channel_text)?;
    let uploads_playlist_id = &channel_resp.items[0]
        .content_details
        .as_ref()
        .unwrap()
        .related_playlists
        .uploads;
    //println!("Uploads playlist id: {}", uploads_playlist_id);

    // Iterate through all pages of playlistItems
    let mut next_page_token: Option<String> = None;
    let mut total_likes = 0u64;
    let mut total_views = 0u64;
    let mut video_count = 0u64;
    let mut videos: Vec<VideoData> = Vec::new();

    loop {
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/playlistItems?part=contentDetails&playlistId={}&maxResults=50&key={}",
            uploads_playlist_id, api_key
        );
        if let Some(token) = &next_page_token {
            url.push_str(&format!("&pageToken={}", token));
        }
        //println!("PlaylistItems URL: {}", url);
        let playlist_text = client.get(&url).send().await?.text().await?;
        //println!("PlaylistItems raw JSON: {}", playlist_text);
        let playlist_resp: PlaylistResponse = serde_json::from_str(&playlist_text)?;

        for item in playlist_resp.items {
            let video_id = item.content_details.video_id;
            let url = format!(
                "https://www.googleapis.com/youtube/v3/videos?part=snippet,statistics&id={}&key={}",
                video_id, api_key
            );
            //println!("Video URL: {}", url);
            let video_text = client.get(&url).send().await?.text().await?;
            //println!("Video raw JSON: {}", video_text);

            // Try to deserialize
            match serde_json::from_str::<VideoResponse>(&video_text) {
                Ok(video_resp) => {
                    if video_resp.items.is_empty() {
                        println!("No items for video {}", video_id);
                        continue;
                    }
                    let video = &video_resp.items[0];
                    let title = video.snippet.title.clone();
                    let published = video.snippet.published_at.clone();
                    let views = video.statistics.view_count.parse::<u64>().unwrap_or(0);
                    let likes = video.statistics.like_count.as_deref().unwrap_or("0").parse::<u64>().unwrap_or(0);

                    total_likes += likes;
                    total_views += views;
                    video_count += 1;

                    videos.push(VideoData { title, published, views, likes });
                }
                Err(e) => {
                    eprintln!("Failed to decode video {}: {}", video_id, e);
                }
            }
        }

        if let Some(token) = playlist_resp.next_page_token {
            next_page_token = Some(token);
        } else {
            break;
        }
    }

    // Sorting
    if let Some(key) = sort_key.as_deref() {
        let order = sort_order.as_deref().unwrap_or("asc");
        match (key, order) {
            ("likes", "asc") => videos.sort_by_key(|v| v.likes),
            ("likes", "desc") => videos.sort_by(|a, b| b.likes.cmp(&a.likes)),
            ("views", "asc") => videos.sort_by_key(|v| v.views),
            ("views", "desc") => videos.sort_by(|a, b| b.views.cmp(&a.views)),
            _ => eprintln!("Unknown sort combination: {} {}", key, order),
        }
    }

    for v in &videos {
        println!(
            "Title: {}\nPublished: {}\nViews: {}\nLikes: {}\n",
            v.title, v.published, v.views, v.likes
        );
    }

    println!("Processed {} videos", video_count);
    println!("Total likes across all videos: {}", total_likes);
    println!("Total views across all videos: {}", total_views);

    Ok(())
}


async fn run_subscriptions(oauth_token: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = "https://www.googleapis.com/youtube/v3/subscriptions?part=snippet&mine=true";
    let resp: serde_json::Value = client
        .get(url)
        .bearer_auth(oauth_token)
        .send()
        .await?
        .json()
        .await?;

    if let Some(items) = resp.get("items").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(title) = item["snippet"]["title"].as_str() {
                println!("title: {}", title);
            }
        }
    } else {
        eprintln!("No subscriptions found or invalid token.");
    }

    Ok(())
}

async fn run_channel_meta(channel_handle: String, api_key: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let query = if channel_handle.starts_with('@') {
        channel_handle.clone()
    } else {
        format!("@{}", channel_handle)
    };

    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=channel&q={}&key={}",
        query, api_key
    );
    let search_resp: SearchResponse = client.get(&url).send().await?.json().await?;
    if search_resp.items.is_empty() {
        eprintln!("No channel found for handle {}", channel_handle);
        return Ok(());
    }
    let channel_id = &search_resp.items[0].id.channel_id;

    let url = format!(
        "https://www.googleapis.com/youtube/v3/channels?part=snippet,statistics&id={}&key={}",
        channel_id, api_key
    );
    let channel_resp: ChannelResponse = client.get(&url).send().await?.json().await?;
    if channel_resp.items.is_empty() {
        eprintln!("No metadata found for channel {}", channel_handle);
        return Ok(());
    }
    let channel = &channel_resp.items[0];

    if let Some(snippet) = &channel.snippet {
        println!("Channel Title: {}", snippet.title);
        println!("Description: {}", snippet.description);
        println!("Published At: {}", snippet.published_at);
    }

    if let Some(stats) = &channel.statistics {
        println!("Subscribers: {}", stats.subscriber_count);
        println!("Total Views: {}", stats.view_count);
        println!("Video Count: {}", stats.video_count);
    }

    Ok(())
}

async fn run_playlists(channel_handle: String, api_key: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let query = if channel_handle.starts_with('@') {
        channel_handle.clone()
    } else {
        format!("@{}", channel_handle)
    };

    // Resolve handle -> channelId
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=channel&q={}&key={}",
        query, api_key
    );
    let search_resp: SearchResponse = client.get(&url).send().await?.json().await?;
    if search_resp.items.is_empty() {
        eprintln!("No channel found for handle {}", channel_handle);
        return Ok(());
    }
    let channel_id = &search_resp.items[0].id.channel_id;

    // Paginate through all public playlists
    let mut next_page_token: Option<String> = None;
    let mut all_playlists: Vec<PlaylistListItem> = Vec::new();

    loop {
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/playlists?part=snippet&channelId={}&maxResults=50&key={}",
            channel_id, api_key
        );
        if let Some(token) = &next_page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let playlists_resp: PlaylistListResponse = client.get(&url).send().await?.json().await?;
        all_playlists.extend(playlists_resp.items);

        if let Some(token) = playlists_resp.next_page_token {
            next_page_token = Some(token);
        } else {
            break;
        }
    }

    if all_playlists.is_empty() {
        eprintln!("No public playlists found for channel {}", channel_handle);
        return Ok(());
    }

    println!("Public Playlists for {}:", channel_handle);
    for pl in &all_playlists {
        println!("Title: {}\nDescription: {}\n", pl.snippet.title, pl.snippet.description);
    }
    println!("Total playlists: {}", all_playlists.len());

    Ok(())
}

async fn run_search_videos(
    channel_handle: String,
    api_key: String,
    query: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Resolve handle -> channelId
    let handle_query = if channel_handle.starts_with('@') {
        channel_handle.clone()
    } else {
        format!("@{}", channel_handle)
    };
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=channel&q={}&key={}",
        handle_query, api_key
    );
    //println!("Channel search URL: {}", url);
    let search_text = client.get(&url).send().await?.text().await?;
    //println!("Channel search raw JSON:\n{}", search_text);

    let search_resp: SearchResponse = match serde_json::from_str(&search_text) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to decode channel search: {}", e);
            println!("Channel search raw JSON:\n{}", search_text);
            return Ok(());
        }
    };
    if search_resp.items.is_empty() {
        eprintln!("No channel found for handle {}", channel_handle);
        return Ok(());
    }
    let channel_id = &search_resp.items[0].id.channel_id;
    //println!("Resolved channel_id: {}", channel_id);

    // Paginate through all search results
    let mut next_page_token: Option<String> = None;
    let mut all_results: Vec<VideoSearchItem> = Vec::new();

    loop {
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&channelId={}&q={}&maxResults=5&key={}",
            channel_id, query, api_key
        );
        if let Some(token) = &next_page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        //println!("Video search URL: {}", url);
        let resp_text = client.get(&url).send().await?.text().await?;
        //println!("Video search raw JSON:\n{}", resp_text);

        // Try to decode, but don’t crash if it fails
        match serde_json::from_str::<VideoSearchResponse>(&resp_text) {
            Ok(page_resp) => {
                all_results.extend(page_resp.items);
                next_page_token = page_resp.next_page_token;
                if next_page_token.is_none() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to decode video search page: {}", e);
                break;
            }
        }
    }

    println!("Total raw items collected: {}", all_results.len());
    for item in &all_results {
        println!(
            "Title: {}\nPublished: {}\nDescription: {}\nVideo ID: {}\n",
            item.snippet.title,
            item.snippet.published_at,
            item.snippet.description,
            item.id.video_id
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Command::from_args();
    match cmd {
        Command::Stats {
            channel_handle,
            api_key,
            sort_key,
            sort_order,
        } => run_stats(channel_handle, api_key, sort_key, sort_order).await?,
        Command::DumpSubs { oauth_token } => run_subscriptions(oauth_token).await?,
        Command::ChannelMeta { channel_handle, api_key } => run_channel_meta(channel_handle, api_key).await?,
        Command::Playlists { channel_handle, api_key } => run_playlists(channel_handle, api_key).await?,
        Command::SearchVideos { channel_handle, api_key, query } =>
            run_search_videos(channel_handle, api_key, query).await?,
    }
    Ok(())
}
