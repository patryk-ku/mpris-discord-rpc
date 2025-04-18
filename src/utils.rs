use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use pickledb::PickleDb;
use reqwest;
use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use serde_json;
use std::{env, fs, process};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// Use to print debug log if enabled with argument
#[macro_export]
macro_rules! debug_log {
    ($cond:expr, $($arg:tt)*) => {
        if $cond {
            println!("\x1b[34;1m[debug]\x1b[0m {}", format!($($arg)*));
        }
    };
}

fn is_systemd_present() {
    match process::Command::new("ps")
        .arg("-p")
        .arg("1")
        .arg("-o")
        .arg("comm=")
        .output()
    {
        Ok(output) => {
            let init_process = String::from_utf8_lossy(&output.stdout);
            if init_process.trim() == "systemd" {
                return;
            }
            println!("\x1b[33;1mINFO: Most likely systemd is not available on your system.\x1b[0m");
        }
        Err(_) => {
            println!(
                "\x1b[33;1mINFO: Could not detect if systemd is available on your system.\x1b[0m"
            )
        }
    }
    println!(
        "You can try using XDG Autostart instead to add the application to autostart without systemd."
    );
    println!("Use the \x1b[32;1m--xdg\x1b[0m flag with the subcommands like this: \x1b[32;1mmpris-discord-rpc enable --xdg\x1b[0m.");
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
            is_systemd_present();
            process::exit(1);
        }
    }

    match process::Command::new("systemctl")
        .arg("--user")
        .arg("enable")
        .arg("--now")
        .arg("mpris-discord-rpc.service")
        .status()
    {
        Ok(_) => println!("Enabled and started user systemd service."),
        Err(_) => {
            println!("Failed to enable and start user systemd service.");
            is_systemd_present();
            process::exit(1);
        }
    }

    process::exit(0);
}

pub fn disable_service() {
    match process::Command::new("systemctl")
        .arg("--user")
        .arg("disable")
        .arg("--now")
        .arg("mpris-discord-rpc.service")
        .status()
    {
        Ok(_) => println!("Stopped and disabled user systemd service."),
        Err(_) => {
            println!("Failed to stop and disable user systemd service.");
            is_systemd_present();
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

pub fn get_config_path() -> Option<std::path::PathBuf> {
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        Some(std::path::PathBuf::from(config_home))
    } else if let Some(home_dir) = env::var_os("HOME") {
        let mut path = std::path::PathBuf::from(home_dir);
        path.push(".config");
        Some(path)
    } else {
        None
    }
}

pub fn add_xdg_autostart() {
    let mut desktopt_file_path = match get_config_path() {
        Some(path) => path,
        None => {
            println!("\x1b[31mWARNING: Failed to determine user config directory.\x1b[0m");
            process::exit(1);
        }
    };
    desktopt_file_path.push("autostart");
    desktopt_file_path.push("mpris-discord-rpc.desktop");

    let desktop_file_content = r#"[Desktop Entry]
Name=mpris-discord-rpc
Type=Application
Exec=mpris-discord-rpc
X-GNOME-Autostart-enabled=true
Hidden=false
StartupNotify=false
Terminal=false
"#;

    match fs::write(&desktopt_file_path, desktop_file_content) {
        Ok(_) => {
            println!(
                "Created file: \x1b[32;1m{}\x1b[0m ",
                desktopt_file_path.display()
            );
            println!("This RPC should now start automatically with your system if your DE/WM supports XDG Autostart.");
        }
        Err(_) => {
            println!("\x1b[31mERROR: Failed to create autostart .desktop file.\x1b[0m");
            process::exit(1);
        }
    }

    process::exit(0);
}

pub fn remove_xdg_autostart() {
    let mut desktopt_file_path = match get_config_path() {
        Some(path) => path,
        None => {
            println!("\x1b[31mWARNING: Failed to determine user config directory.\x1b[0m");
            process::exit(1);
        }
    };
    desktopt_file_path.push("autostart");
    desktopt_file_path.push("mpris-discord-rpc.desktop");

    match fs::remove_file(&desktopt_file_path) {
        Ok(_) => println!(
            "Removed file: \x1b[32;1m{}\x1b[0m ",
            desktopt_file_path.display()
        ),
        Err(_) => {
            println!("\x1b[31mERROR: Failed to remove autostart .desktop file.\x1b[0m");
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
    album: &str,
    mut _cover_url: String,
    cache_enabled: bool,
    album_cache: &mut PickleDb,
    artist: &str,
    lastfm_api_key: &str,
) -> String {
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

    let request_url = format!(
    	"http://ws.audioscrobbler.com/2.0/?method=album.getinfo&api_key={}&artist={}&album={}&autocorrect=0&format=json",
     	lastfm_api_key,
     	url_escape::encode_component(artist),
     	url_escape::encode_component(album)
    );

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

pub fn get_cover_url_musicbrainz(
    album_id: &str,
    album: &str,
    mut _cover_url: String,
    cache_enabled: bool,
    album_cache: &mut PickleDb,
    artist: &str,
) -> String {
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

    let user_agent = format!(
        "mpris-discord-rpc/{} (patryk.kurdziel@protonmail.com)",
        VERSION
    );

    let request_url = format!(
    	"https://musicbrainz.org/ws/2/release/?query=artist:\"{}\"ANDrelease:\"{}\"&fmt=json&limit=1",
    	url_escape::encode_component(artist),
     	url_escape::encode_component(album)
    );

    let client = Client::new();
    let mut mbid: String = match client
        .get(request_url)
        .header(USER_AGENT, &user_agent)
        .send()
    {
        Ok(res) => match res.json::<serde_json::Value>() {
            Ok(data) => data["releases"][0]["id"].to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    if !mbid.is_empty() && (mbid.len() > 5) {
        mbid.pop();
        mbid.remove(0);
    }

    let mut url: String = match client
        .get(format!("http://coverartarchive.org/release/{}/", mbid))
        .header(USER_AGENT, &user_agent)
        .send()
    {
        Ok(res) => match res.json::<serde_json::Value>() {
            Ok(data) => data["images"][0]["thumbnails"]["small"].to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    if !url.is_empty() && (url.len() > 5) {
        url.pop();
        url.remove(0);
        println!("[listenbrainz] fetched image link: {}", url);

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
