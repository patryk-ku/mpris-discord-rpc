use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use pickledb::PickleDb;
use reqwest;
use serde_json;
use std::process;

// Use to print debug log if enabled with argument
#[macro_export]
macro_rules! debug_log {
    ($cond:expr, $($arg:tt)*) => {
        if $cond {
            println!("\x1b[34;1m[debug]\x1b[0m {}", format!($($arg)*));
        }
    };
}

pub fn enable_service() {
    match process::Command::new("systemctl")
        .arg("--user")
        .arg("daemon-reload")
        .status()
    {
        Ok(_) => println!("Reloaded user systemd services."),
        Err(_) => {
            println!("Failed to reload user systemd services.");
            process::exit(1);
        }
    }

    match process::Command::new("systemctl")
        .arg("--user")
        .arg("enable")
        .arg("mpris-discord-rpc.service")
        .status()
    {
        Ok(_) => println!("Enabled user systemd service."),
        Err(_) => {
            println!("Failed to enable user systemd service.");
            process::exit(1);
        }
    }

    match process::Command::new("systemctl")
        .arg("--user")
        .arg("start")
        .arg("mpris-discord-rpc.service")
        .status()
    {
        Ok(_) => println!("Started user systemd service."),
        Err(_) => {
            println!("Failed to start user systemd service.");
            process::exit(1);
        }
    }

    process::exit(0);
}

pub fn disable_service() {
    match process::Command::new("systemctl")
        .arg("--user")
        .arg("stop")
        .arg("mpris-discord-rpc.service")
        .status()
    {
        Ok(_) => println!("Stopped user systemd service."),
        Err(_) => {
            println!("Failed to stop user systemd service.");
            process::exit(1);
        }
    }

    match process::Command::new("systemctl")
        .arg("--user")
        .arg("disable")
        .arg("mpris-discord-rpc.service")
        .status()
    {
        Ok(_) => println!("Disabled user systemd service."),
        Err(_) => {
            println!("Failed to disable user systemd service.");
            process::exit(1);
        }
    }

    process::exit(0);
}

pub fn restart_service() {
    match process::Command::new("systemctl")
        .arg("--user")
        .arg("restart")
        .arg("mpris-discord-rpc.service")
        .status()
    {
        Ok(_) => println!("Restarted user systemd service."),
        Err(_) => {
            println!("Failed to restart user systemd service.");
            process::exit(1);
        }
    }
    process::exit(0);
}

pub fn clear_activity(is_activity_set: &mut bool, client: &mut DiscordIpcClient) {
    if *is_activity_set {
        let is_activity_cleared = client.clear_activity().is_ok();

        if is_activity_cleared {
            *is_activity_set = false;
            return;
        }

        let is_reconnected = client.reconnect().is_ok();
        if !is_reconnected {
            return;
        }

        if client.clear_activity().is_ok() {
            *is_activity_set = false;
        }
    }
}

pub fn get_cover_url(
    album_id: &str,
    last_album_id: &str,
    album: &str,
    mut _cover_url: String,
    cache_enabled: bool,
    album_cache: &mut PickleDb,
    artist: &str,
    lastfm_api_key: &str,
) -> String {
    if album_id == last_album_id {
        return _cover_url;
    }

    // If no album or Unknown Album
    if album.eq("Unknown Album") {
        println!("Missing album name or Unknown Album.");

        return String::from("missing-cover");
    }

    // Load from cache if enabled
    if cache_enabled {
        let cache_url = if album_cache.exists(&album_id) {
            match album_cache.get(&album_id) {
                Some(url) => url,
                None => String::new(),
            }
        } else {
            String::new()
        };

        if (!cache_url.is_empty()) & (cache_url.len() > 5) {
            return String::from(cache_url);
        }
    }

    let request_url = format!("http://ws.audioscrobbler.com/2.0/?method=album.getinfo&api_key={}&artist={}&album={}&autocorrect=0&format=json", lastfm_api_key, url_escape::encode_component(artist), url_escape::encode_component(album));

    let mut url: String = match reqwest::blocking::get(request_url) {
        Ok(res) => match res.json::<serde_json::Value>() {
            Ok(data) => data["album"]["image"][3]["#text"].to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    if !url.is_empty() && (url.len() > 5) {
        url.pop();
        url.remove(0);
        println!("[last.fm] fetched image link: {}", url);

        // Save cover url to cache
        if cache_enabled {
            match album_cache.set(&album_id, &url) {
                Ok(_) => {
                    println!("[cache] saved image url for: {}.", album_id)
                }
                Err(_) => {
                    println!("[cache] error, unable to write to cache file.")
                }
            }
        }

        return url;
    }

    return String::from("missing-cover");
}

pub fn get_lastfm_avatar(username: &str, lastfm_api_key: &str) -> String {
    let request_url = format!(
        "http://ws.audioscrobbler.com/2.0/?method=user.getinfo&api_key={}&user={}&format=json",
        lastfm_api_key,
        url_escape::encode_component(username)
    );

    let mut url: String = match reqwest::blocking::get(request_url) {
        Ok(res) => match res.json::<serde_json::Value>() {
            Ok(data) => data["user"]["image"][3]["#text"].to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    if !url.is_empty() && (url.len() > 15) {
        url.pop();
        url.remove(0);
        println!("[last.fm] fetched avatar link: {}", url);
        return url;
    }

    return String::new();
}

pub fn sanitize_name(input: &str) -> String {
    input
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

// pub fn format_time(seconds: u64) -> String {
//     let minutes = seconds / 60;
//     let remaining_seconds = seconds % 60;
//     format!("{:02}:{:02}", minutes, remaining_seconds)
// }
