use clap::Parser;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use dotenvy_macro::dotenv;
use mpris::PlayerFinder;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use reqwest;
use serde_json;
use url_escape;

use std::env;
use std::ops::Sub;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Activity refresh rate (min 5)
    #[arg(short, long, value_name = "seconds", default_value_t = 10, value_parser = clap::value_parser!(u64).range(5..))]
    interval: u64,

    /// Display "Open user's last.fm profile" button
    #[arg(short, long, value_name = "nickname", value_parser = clap::value_parser!(String))]
    profile_button: Option<String>,

    /// Display "Search this song on YouTube" button
    #[arg(short, long)]
    yt_button: bool,

    /// Disable cache (not recommended)
    #[arg(short, long)]
    disable_cache: bool,

    /// Displays all available player names and exits. Use to get your players name for -n parameter
    #[arg(short, long)]
    list_players: bool,

    /// Get status only from given player name. Use -l to get player exact name to use with this parameter
    #[arg(short = 'n', long, value_name = "Player Name", value_parser = clap::value_parser!(String))]
    player_name: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Load api key from .env file durning compilation
    const LASTFM_API_KEY: &str = dotenv!("LASTFM_API_KEY");

    // User settings parsed from args:
    // Refresh rate (sleep after every loop)
    let interval: u64 = args.interval; // important: min 5 sec
    println!("Refresh rate: {} seconds", interval);

    // Display "Search this song on YouTube" button under activity
    let show_yt_link: bool = args.yt_button;
    println!("YT button: {}", show_yt_link);

    // Display "Open user's last.fm profile" button under activity
    let mut lastfm_nickname: String = String::new();
    let show_lastfm_link = match args.profile_button {
        Some(nick) => {
            lastfm_nickname = nick;
            true
        }
        None => false,
    };
    println!("Profile button: {}", show_lastfm_link);
    if show_lastfm_link {
        println!("Nickname set: {}", lastfm_nickname);
    }

    // Enable/disable use of cache
    let cache_enabled: bool = !args.disable_cache;
    println!("Cache: {}", cache_enabled);

    // List available players and exit
    let list_players: bool = args.list_players;

    // Get status only from player with given name
    let mut player_name: String = String::new();
    let player_name_enabled: bool = match args.player_name {
        Some(name) => {
            player_name = name;
            true
        }
        None => false,
    };

    // Vars for activity update detection
    let mut last_title: String = String::new();
    let mut last_album: String = String::new();
    let mut last_artist: String = String::new();
    let mut last_album_id: String = String::new();
    let mut last_track_position: u64 = 0;
    let mut last_is_playing: bool = false;

    let mut _cover_url: String = "".to_string();
    let mut is_first_time: bool = true;
    let mut is_interrupted: bool = false;
    let mut is_activity_set: bool = false;

    // Preventing stdout spam while waiting for player or discord
    let mut dbus_notif: bool = false;
    let mut player_notif: u8 = 0;
    let mut discord_notif: bool = false;

    let mut client = DiscordIpcClient::new("1129859263741837373")?;

    // Set cache path
    let home_dir = match env::var("HOME") {
        Ok(val) => PathBuf::from(val),
        Err(_) => PathBuf::from("~/"),
    };
    let cache_dir = match env::var("XDG_CACHE_HOME") {
        Ok(val) => {
            let mut tmp_path = PathBuf::from(val);
            tmp_path.push("mpris-discord-rpc");
            tmp_path
        }
        Err(_) => {
            let mut tmp_path = PathBuf::from(home_dir);
            tmp_path.push(".cache");
            tmp_path.push("mpris-discord-rpc");
            tmp_path
        }
    };

    if cache_enabled {
        println!("Cache location: {}", &cache_dir.display());
    }

    // Cache file
    let mut db_path = PathBuf::from(&cache_dir);
    db_path.push("album_cache.db");

    let mut album_cache = match PickleDb::load(
        &db_path,
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    ) {
        Ok(db) => {
            if cache_enabled {
                println!("Cache loaded from file: {}", &db_path.display());
            }
            db
        }
        Err(_) => {
            if cache_enabled {
                println!("Generated new cache file: {}", &db_path.display());
            }
            PickleDb::new(
                &db_path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Json,
            )
        }
    };

    loop {
        // Connect to MPRIS2
        let player = match PlayerFinder::new() {
            Ok(a) => {
                dbus_notif = false;
                a
            }
            Err(_) => {
                if !dbus_notif {
                    println!("Could not connect to D-Bus.");
                    dbus_notif = true;
                }
                sleep(Duration::from_secs(interval));
                continue;
            }
        };

        // List available players and exit
        if list_players {
            match player.find_all() {
                Ok(player_list) => {
                    if player_list.is_empty() {
                        println!("Could not find any player with MPRIS2 support.");
                    } else {
                        println!("────────────────────────────────────────────────────");
                        println!("List of available music players with MPRIS2 support:");
                        for music_player in &player_list {
                            println!(" - {}", music_player.identity());
                        }
                        println!("────────────────────────────────────────────────────");
                        println!("Use the name to choose from which source the script should take data for the discord status.");
                        println!("Usage instruction:");
                        println!(r#" ./mpris-discord-rpc -n "{}""#, player_list[0].identity());
                    }
                }
                Err(_) => {
                    println!("Could not find any player with MPRIS2 support.");
                }
            };
            return Ok(());
        }

        // Get player by name if enabled
        let player = if player_name_enabled {
            player.find_by_name(&player_name)
        } else {
            player.find_active()
        };

        // Find acive player
        let player = match player {
            Ok(a) => {
                if player_notif != 1 {
                    println!("Found active player with MPRIS2 support.");
                    player_notif = 1;
                }
                a
            }
            // Err(mpris::FindingError::NoPlayerFound) => {}
            Err(_) => {
                if player_notif != 2 {
                    println!(
                        "Could not find any player with MPRIS2 support. Waiting for any player..."
                    );
                    player_notif = 2;
                    discord_notif = false;
                }
                is_interrupted = true;

                if is_activity_set {
                    match client.reconnect() {
                        Ok(res) => {
                            client.clear_activity()?;
                            is_activity_set = false;
                            res
                        }
                        Err(_) => (),
                    };
                }

                sleep(Duration::from_secs(interval));
                continue;
            }
        };

        // Connect with Discord
        if is_first_time {
            match client.connect() {
                Ok(a) => {
                    println!("Connected to Discord.");
                    discord_notif = false;
                    a
                }
                Err(_) => {
                    if !discord_notif {
                        println!("Could not connect to Discord. Waiting for discord to start...");
                        discord_notif = true;
                    }
                    sleep(Duration::from_secs(interval));
                    continue;
                }
            };
            is_first_time = false;
        } else {
            match client.reconnect() {
                Ok(a) => {
                    println!("Reconnected to Discord.");
                    is_interrupted = true;
                    discord_notif = false;
                    a
                }
                Err(_) => {
                    if !discord_notif {
                        println!("Could not reconnect to Discord. Waiting for discord to start...");
                        discord_notif = true;
                    }
                    sleep(Duration::from_secs(interval));
                    continue;
                }
            };
        }

        loop {
            // Get metadata from player
            let metadata = match player.get_metadata() {
                Ok(a) => a,
                Err(_) => {
                    println!("Could not get metadata from player");

                    if is_activity_set {
                        match client.reconnect() {
                            Ok(res) => {
                                client.clear_activity()?;
                                is_activity_set = false;
                                res
                            }
                            Err(_) => (),
                        };
                    }
                    break;
                }
            };
            // println!("{:#?}", metadata);

            let playback_status = match player.get_playback_status() {
                Ok(status) => status,
                Err(_) => {
                    println!("Could not get playback status from player");

                    if is_activity_set {
                        match client.reconnect() {
                            Ok(res) => {
                                client.clear_activity()?;
                                is_activity_set = false;
                                res
                            }
                            Err(_) => (),
                        };
                    }
                    break;
                }
            };

            let is_playing: bool = match playback_status {
                mpris::PlaybackStatus::Playing => true,
                mpris::PlaybackStatus::Paused => false,
                mpris::PlaybackStatus::Stopped => false,
            };
            // println!("{:#?}", playback_status);

            // Parse metadata
            let title = metadata.title().unwrap_or("Unknown Title");
            let mut album = metadata.album_name().unwrap_or("Unknown Album");
            if album.is_empty() {
                album = "Unknown Album";
            }
            let artist = metadata.artists().unwrap_or(vec!["Unknown Artist"]);
            let artist = artist[0];
            let album_id = format!("{} - {}", artist, album);

            let mut metadata_changed: bool = false;

            // println!("{title} - {last_title}");
            // println!("{album} - {last_album}");
            // println!("{artist} - {last_artist}");
            // println!("{is_playing} - {last_is_playing}");
            if (title != last_title)
                | (album != last_album)
                | (artist != last_artist)
                | (is_playing != last_is_playing)
            {
                metadata_changed = true;
            }

            // TODO: Handle unknown metadata
            // If all metadata values are unknown then skip
            // if (artist == "Unknown Artist") & (album == "Unknown Album") & (title == "Unknown Title") {
            //     println!("Unknown metadata, skipping...");
            //     sleep(Duration::from_secs(interval));
            //     break;
            // }

            // Get track position if supported by player else return 0 secs
            let track_position = match player.get_position() {
                Ok(position) => position,
                Err(_) => Duration::new(0, 0),
            };
            // println!("Track position: {:#?}", track_position);

            // Check if song repeated
            if track_position.as_secs() < last_track_position {
                metadata_changed = true;
            }

            if !metadata_changed & !is_interrupted {
                // println!("[debug] same metadata and status, skipping...");
                sleep(Duration::from_secs(interval));
                continue;
            }

            // Get unix time of track start if supported, else return time now
            let last_time: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => n.sub(track_position).as_secs(),
                Err(_) => 0,
            };
            let track_position = track_position.as_secs();

            // Fetch cover from last.fm
            if !album_id.eq(&last_album_id) {
                // If no album or Unknown Album
                if album.eq("Unknown Album") {
                    println!("Missing album name or Unknown Album.");
                    _cover_url = "missing-cover".to_string();
                } else {
                    // Load from cache if enabled
                    let mut cache_url: String = String::new();
                    if cache_enabled {
                        cache_url = if album_cache.exists(&album_id) {
                            match album_cache.get(&album_id) {
                                Some(url) => url,
                                None => String::new(),
                            }
                        } else {
                            String::new()
                        };
                    }

                    if (!cache_url.is_empty()) & (cache_url.len() > 5) {
                        // println!("Cached image link: {cache_url}");
                        _cover_url = cache_url.to_string();
                    } else {
                        let request_url = format!("http://ws.audioscrobbler.com/2.0/?method=album.getinfo&api_key={}&artist={}&album={}&autocorrect=0&format=json", LASTFM_API_KEY, url_escape::encode_component(artist), url_escape::encode_component(album));
                        // println!("{}", request_url);

                        _cover_url = match reqwest::blocking::get(request_url) {
                            Ok(res) => match res.json::<serde_json::Value>() {
                                Ok(data) => {
                                    let mut url = data["album"]["image"][3]["#text"].to_string();
                                    if (!url.is_empty()) & (url.len() > 5) {
                                        url.pop();
                                        url.remove(0);

                                        // Save cover url to cache
                                        if cache_enabled {
                                            match album_cache.set(&album_id, &url.to_string()) {
                                                Ok(_) => println!("[cache] fetched and saved image url for: {album_id}."),
                                                // _ => (),
                                                Err(_) => println!("[cache] error, unable to write to cache file."),
                                            }
                                        }
                                    } else {
                                        url = "missing-cover".to_string();
                                    }
                                    url
                                }
                                Err(_) => "missing-cover".to_string(),
                            },
                            Err(_) => "missing-cover".to_string(),
                        };

                        println!("Fetched image link: {_cover_url}");
                    }
                }
            }

            let image: String = if _cover_url.is_empty() {
                "missing-cover".to_string()
            } else {
                _cover_url.clone()
            };

            // Save last refresh info
            last_title = title.to_string();
            last_album = album.to_string();
            last_artist = artist.to_string();
            last_album_id = album_id.to_string();
            last_track_position = track_position;
            last_is_playing = is_playing;

            // Set activity
            let song_name: String = format!("{artist} - {title}");
            let title = format!("{} ", title); // Discord activity min 2 char len bug fix
            let artist = format!("by: {}", artist);
            let album = format!("album: {}", album);
            let status_text: String = if is_playing {
                "playing".to_string()
            } else {
                "paused".to_string()
            };
            let yt_url: String = format!(
                "https://www.youtube.com/results?search_query={}",
                url_escape::encode_component(&song_name)
            );
            let lastfm_url: String = format!(
                "https://www.last.fm/user/{}",
                url_escape::encode_component(&lastfm_nickname)
            );

            let payload = activity::Activity::new()
                .state(&artist)
                .details(&title)
                .assets(
                    activity::Assets::new()
                        .large_image(&image)
                        .small_image(&status_text)
                        .large_text(&album)
                        .small_text(&status_text),
                )
                // .buttons(vec![
                //     activity::Button::new("Search this song on YouTube", &yt_url),
                //     activity::Button::new("Open user's last.fm profile", &lastfm_url),
                // ])
                ;

            let payload = if is_playing {
                payload.timestamps(activity::Timestamps::new().start(last_time.try_into().unwrap()))
            } else {
                payload.timestamps(activity::Timestamps::new().end(last_time.try_into().unwrap()))
            };

            let mut buttons = Vec::new();
            if show_yt_link {
                buttons.push(activity::Button::new(
                    "Search this song on YouTube",
                    &yt_url,
                ));
            }
            if show_lastfm_link {
                buttons.push(activity::Button::new(
                    "Open user's last.fm profile",
                    &lastfm_url,
                ));
            }
            let payload = if buttons.len() > 0 {
                payload.buttons(buttons)
            } else {
                payload
            };

            match client.set_activity(payload) {
                Ok(a) => {
                    is_interrupted = false;
                    is_activity_set = true;
                    println!("=> Set activity [{status_text}]: {song_name}");
                    a
                }
                Err(_) => {
                    println!("Could not set activity.");
                    is_interrupted = true;
                    is_activity_set = false;
                    client.close()?;
                    break;
                }
            };

            sleep(Duration::from_secs(interval));
        }

        sleep(Duration::from_secs(interval));
    }
}
