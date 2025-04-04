use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use dotenvy_macro::dotenv;
use mpris::PlayerFinder;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use url_escape;

use std::env;
use std::fs;
use std::ops::Sub;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

mod settings;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load api key from .env file durning compilation
    const LASTFM_API_KEY: &str = dotenv!("LASTFM_API_KEY");

    // Set home path, If $HOME is not set, do not write or read anything from the user's disk
    let (home_exists, home_dir) = match env::var("HOME") {
        Ok(val) => (true, PathBuf::from(val)),
        Err(_) => (false, PathBuf::from("/")),
    };

    let settings = settings::load_settings();

    debug_log!(settings.debug_log, "Settings: {:#?}", settings);
    debug_log!(settings.debug_log, "home_exists: {}", home_exists);
    debug_log!(settings.debug_log, "home_dir: {}", home_dir.display());

    // Exec subcommands
    match settings.suboptions.command {
        Some(settings::Commands::Enable {}) => utils::enable_service(&home_dir),
        Some(settings::Commands::Disable {}) => utils::disable_service(),
        Some(settings::Commands::Restart {}) => utils::restart_service(),
        None => {}
    }

    // User settings
    // Main loop interval
    let mut interval = settings.interval.unwrap_or(10);
    if interval < 5 {
        interval = 5
    }
    debug_log!(settings.debug_log, "interval: {}", interval);

    // Nicknames for buttons
    let lastfm_name = settings.lastfm_name.unwrap_or_default();
    let listenbrainz_name = settings.listenbrainz_name.unwrap_or_default();

    let small_image = settings.small_image.unwrap_or(String::from("playPause"));
    let mut lastfm_avatar = String::new();
    if small_image == "lastfmAvatar" && !lastfm_name.is_empty() {
        lastfm_avatar = utils::get_lastfm_avatar(&lastfm_name, LASTFM_API_KEY);
        debug_log!(settings.debug_log, "lastfm_avatar: {}", lastfm_avatar);
    }
    let lastfm_icon_text = if !lastfm_name.is_empty() {
        lastfm_name.to_string() + " on Last.fm"
    } else {
        String::new()
    };

    // Force player id and name
    let force_player_name = settings.force_player_name.unwrap_or_default();
    let force_player_id = settings.force_player_id.unwrap_or_default();

    // Enable/disable use of cache
    let mut cache_enabled: bool = !settings.disable_cache;
    if !home_exists {
        cache_enabled = false;
    }

    // Allowlist of music players
    let allowlist_enabled: bool = match settings.allowlist.len() {
        0 => false,
        _ => true,
    };

    // Vars for activity update detection
    let mut last_title: String = String::new();
    let mut last_album: String = String::new();
    let mut last_artist: String = String::new();
    let mut last_album_artist: String = String::new();
    let mut last_album_id: String = String::new();
    let mut last_track_position: u64 = 0;
    let mut last_is_playing: bool = false;

    let mut _cover_url: String = "".to_string();
    let mut is_first_time_audio: bool = true;
    let mut is_first_time_video: bool = true;
    let mut is_interrupted: bool = false;
    let mut is_activity_set: bool = false;

    // Preventing stdout spam while waiting for player or discord
    let mut dbus_notif: bool = false;
    let mut player_notif: u8 = 0;
    let mut discord_notif: bool = false;

    let mut client_audio = DiscordIpcClient::new("1129859263741837373")?;
    let mut client_video = DiscordIpcClient::new("1356756023813210293")?;
    let mut client: &mut DiscordIpcClient = &mut client_audio;

    // Set cache path
    let cache_dir = match env::var("XDG_CACHE_HOME") {
        Ok(xgd_cache_home) => PathBuf::from(xgd_cache_home).join("mpris-discord-rpc"),
        Err(_) => home_dir.join(".cache/mpris-discord-rpc"),
    };

    if cache_enabled {
        debug_log!(
            settings.debug_log,
            "Cache location: {}",
            &cache_dir.display()
        );
        if let Err(err) = fs::create_dir_all(&cache_dir) {
            println!("Could not create cache directory: {}", err);
        }
    }

    // Cache file
    let db_path = cache_dir.join("album_cache.db");
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
        debug_log!(
            settings.debug_log,
            "───────────────────────────────Loop─1───────────────────────────────────"
        );
        // Connect to MPRIS
        let player = match PlayerFinder::new() {
            Ok(player) => {
                dbus_notif = false;
                player
            }
            Err(err) => {
                if !dbus_notif {
                    println!("Could not connect to D-Bus: {}", err);
                    dbus_notif = true;
                }
                sleep(Duration::from_secs(interval));
                continue;
            }
        };

        // List available players and exit
        if settings.list_players {
            match player.find_all() {
                Ok(player_list) => {
                    if player_list.is_empty() {
                        println!("Could not find any player with MPRIS support.");
                    } else {
                        println!("");
                        println!("────────────────────────────────────────────────────");
                        println!("List of available music players with MPRIS support:");
                        for music_player in &player_list {
                            println!(" * {}", music_player.identity());
                        }
                        println!("");
                        println!("Use the name to choose from which source the script should take data for the Discord status.");
                        println!("Usage instructions:");
                        println!("");
                        println!(r#" mpris-discord-rpc -a "{}""#, player_list[0].identity());
                        println!("");
                        println!("You can use the -a argument multiple times to add more than one player to the allowlist:");
                        println!("");
                        println!(
                            r#" mpris-discord-rpc -a "{}" -a "Second Player" -a "Any other player""#,
                            player_list[0].identity()
                        );
                    }
                }
                Err(_) => {
                    println!("Could not find any player with MPRIS support.");
                }
            };
            return Ok(());
        }

        // Find active player (and filter them by name if enabled)
        let player_finder = if allowlist_enabled {
            let mut allowlist_finder = Err(mpris::FindingError::NoPlayerFound);
            for allowlist_entry in &settings.allowlist {
                allowlist_finder = player.find_by_name(&allowlist_entry);

                if allowlist_finder.is_ok() {
                    break;
                }
            }
            allowlist_finder
        } else {
            player.find_active()
        };

        // Connect with player
        let player = match player_finder {
            Ok(player) => {
                if player_notif != 1 {
                    println!("Found active player with MPRIS support.");
                    player_notif = 1;
                }
                player
            }
            Err(_) => {
                if player_notif != 2 {
                    if allowlist_enabled {
                        println!(
                            "Could not find any active player from your allowlist with MPRIS support. Waiting for any player from your allowlist..."
                        );
                    } else {
                        println!(
                            "Could not find any player with MPRIS support. Waiting for any player..."
                        );
                    }

                    player_notif = 2;
                    discord_notif = false;
                }

                is_interrupted = true;
                utils::clear_activity(&mut is_activity_set, &mut client);
                sleep(Duration::from_secs(interval));
                continue;
            }
        };

        // Use video presence if player is in video_players list
        let is_video_player = settings
            .video_players
            .iter()
            .any(|player_name| player_name == &player.identity().to_string());
        if is_video_player {
            client = &mut client_video;
            debug_log!(settings.debug_log, "Using video player presence");
        } else {
            client = &mut client_audio;
            debug_log!(settings.debug_log, "Using audio player presence");
        }

        let player_name = if force_player_name.is_empty() {
            player.identity().to_string()
        } else {
            force_player_name.to_string()
        };
        let player_id = if force_player_id.is_empty() {
            utils::sanitize_name(&player_name)
        } else {
            force_player_id.to_string()
        };
        debug_log!(settings.debug_log, "player_name: {}", player_name);
        debug_log!(settings.debug_log, "player_id: {}", player_id);

        // Connect with Discord
        if (is_first_time_audio && !is_video_player) || (is_first_time_video && is_video_player) {
            match client.connect() {
                Ok(_) => {
                    println!("Connected to Discord.");
                    discord_notif = false;
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
            if is_video_player {
                is_first_time_video = false;
            } else {
                is_first_time_audio = false;
            }
        } else {
            match client.reconnect() {
                Ok(_) => {
                    if discord_notif {
                        println!("Reconnected to Discord.");
                    }
                    is_interrupted = true;
                    discord_notif = false;
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
            debug_log!(
                settings.debug_log,
                "───────────────────────────────Loop─2───────────────────────────────────"
            );
            // Get metadata from player
            let metadata = match player.get_metadata() {
                Ok(metadata) => metadata,
                Err(err) => {
                    println!("Could not get metadata from player: {}", err);
                    utils::clear_activity(&mut is_activity_set, &mut client);
                    break;
                }
            };
            debug_log!(settings.debug_log, "{:#?}", metadata);

            let playback_status = match player.get_playback_status() {
                Ok(status) => status,
                Err(err) => {
                    println!("Could not get playback status from player: {}", err);
                    utils::clear_activity(&mut is_activity_set, &mut client);
                    break;
                }
            };

            let is_playing: bool = match playback_status {
                mpris::PlaybackStatus::Playing => true,
                mpris::PlaybackStatus::Paused => false,
                mpris::PlaybackStatus::Stopped => false,
            };
            // println!("{:#?}", playback_status);
            debug_log!(
                settings.debug_log,
                "playback_status: {:#?}",
                playback_status
            );

            // Parse metadata
            let title = metadata.title().unwrap_or("Unknown Title");
            let mut album = metadata.album_name().unwrap_or("Unknown Album");
            if album.is_empty() {
                album = "Unknown Album";
            }
            let artist = match metadata.artists() {
                Some(artists) => {
                    if artists.is_empty() {
                        "Unknown Artist"
                    } else {
                        artists[0]
                    }
                }
                None => "Unknown Artist",
            };
            let mut album_artist = match metadata.album_artists() {
                Some(artists) => {
                    if artists.is_empty() {
                        "Unknown Artist"
                    } else {
                        artists[0]
                    }
                }
                None => "Unknown Artist",
            };
            if album_artist.is_empty() || album_artist == "Unknown Artist" {
                album_artist = artist;
            }
            let album_id = format!("{} - {}", album_artist, album);

            // If all metadata values are unknown then break
            if (artist == "Unknown Artist")
                & (album == "Unknown Album")
                & (title == "Unknown Title")
            {
                debug_log!(settings.debug_log, "Unknown metadata, skipping...");
                sleep(Duration::from_secs(interval));
                break;
            }

            // If artist or track is empty then break
            if (artist.len() == 0) | (title.len() == 0) {
                debug_log!(settings.debug_log, "Unknown metadata, skipping...");
                sleep(Duration::from_secs(interval));
                break;
            }

            let mut metadata_changed: bool = false;
            debug_log!(settings.debug_log, "Checking if metadata changed:");
            debug_log!(settings.debug_log, "{title} - {last_title}");
            debug_log!(settings.debug_log, "{album} - {last_album}");
            debug_log!(settings.debug_log, "{artist} - {last_artist}");
            debug_log!(settings.debug_log, "{album_artist} - {last_album_artist}");
            debug_log!(
                settings.debug_log,
                "is_playing: {} - {}",
                is_playing,
                last_is_playing
            );
            if (title != last_title)
                | (album != last_album)
                | (artist != last_artist)
                | (album_artist != last_album_artist)
                | (is_playing != last_is_playing)
            {
                metadata_changed = true;
            }

            // Get track duration if supported by player else return 0
            let track_duration = metadata.length().unwrap_or(Duration::new(0, 0)).as_secs();

            // Get track position if supported by player else return 0 secs
            let mut is_track_position: bool = false;
            let track_position = match player.get_position() {
                Ok(position) => {
                    is_track_position = true;
                    position.as_secs()
                }
                Err(_) => Duration::new(0, 0).as_secs(),
            };
            debug_log!(
                settings.debug_log,
                "track_position: {} - {}",
                track_position,
                last_track_position
            );

            // Check if song repeated
            if (track_position < last_track_position) && !metadata_changed {
                debug_log!(settings.debug_log, "Detected a potential song seek/replay");
                metadata_changed = true;
            }
            last_track_position = track_position; // update it before loop continue
            debug_log!(settings.debug_log, "metadata_changed: {}", metadata_changed);

            if !metadata_changed & !is_interrupted {
                debug_log!(
                    settings.debug_log,
                    "The same metadata and status, skipping..."
                );

                sleep(Duration::from_secs(interval));
                continue;
            }

            // Get unix time of track start if supported, else return time now
            let time_start: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => n.as_secs().sub(track_position),
                Err(_) => 0,
            };

            // Fetch cover from last.fm
            _cover_url = utils::get_cover_url(
                &album_id,
                &last_album_id,
                album,
                _cover_url,
                cache_enabled,
                &mut album_cache,
                album_artist,
                LASTFM_API_KEY,
            );
            let image: String = if _cover_url.is_empty() || _cover_url == "missing-cover" {
                match metadata.art_url() {
                    Some(url) => {
                        if url.starts_with("http") && !settings.disable_mpris_art_url {
                            url.to_string()
                        } else {
                            "missing-cover".to_string()
                        }
                    }
                    _ => "missing-cover".to_string(),
                }
            } else {
                _cover_url.clone()
            };

            // Save last refresh info
            last_title = title.to_string();
            last_album = album.to_string();
            last_artist = artist.to_string();
            last_album_artist = album_artist.to_string();
            last_album_id = album_id.to_string();
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

            let mut assets = activity::Assets::new().large_image(&image);

            if !settings.hide_album_name {
                assets = assets.large_text(&album);
            }

            match small_image.as_str() {
                "player" => {
                    if !settings.disable_mpris_art_url && image.contains("ytimg.com/") {
                        assets = assets.small_image("youtube").small_text("YouTube")
                    } else {
                        assets = assets.small_image(&player_id).small_text(&player_name)
                    }
                }
                "lastfmAvatar" => {
                    if !lastfm_avatar.is_empty() {
                        assets = assets
                            .small_image(&lastfm_avatar)
                            .small_text(&lastfm_icon_text);
                    }
                }
                "none" => {}
                _ => assets = assets.small_image(&status_text).small_text(&status_text),
            }

            // Display paused icon anyway if playpack is paused or stopped
            if status_text != "playing" {
                assets = assets.small_image(&status_text).small_text(&status_text)
            }

            let mut payload = activity::Activity::new()
                .details(&title)
                .assets(assets)
                .activity_type(if is_video_player {
                    activity::ActivityType::Watching
                } else {
                    activity::ActivityType::Listening
                });

            // Don't display Unknown Artist for videos
            if !(is_video_player && artist == "by: Unknown Artist") {
                payload = payload.state(&artist);
            }

            payload = if is_track_position & (track_duration > 0) {
                let time_end = time_start + track_duration;
                if is_playing {
                    payload.timestamps(
                        activity::Timestamps::new()
                            .start(time_start.try_into().unwrap())
                            .end(time_end.try_into().unwrap()),
                    )
                } else {
                    payload.timestamps(
                        activity::Timestamps::new().start(time_start.try_into().unwrap()),
                    )
                }
            } else {
                payload.timestamps(activity::Timestamps::new().end(time_start.try_into().unwrap()))
            };

            // Create urls for activity links
            let yt_url: String = format!(
                "https://www.youtube.com/results?search_query={}",
                url_escape::encode_component(&song_name)
            );
            let lastfm_url: String = format!(
                "https://www.last.fm/user/{}",
                url_escape::encode_component(&lastfm_name)
            );
            let listenbrainz_url: String = format!(
                "https://listenbrainz.org/user/{}/",
                url_escape::encode_component(&listenbrainz_name)
            );
            let mpris_url = match metadata.url() {
                Some(url) => {
                    let url_string = url.to_string();
                    if url_string.starts_with("http://") || url_string.starts_with("https://") {
                        url_string
                    } else {
                        String::new()
                    }
                }
                _ => String::new(),
            };

            // Add activity buttons
            let mut buttons = Vec::new();
            let mut first_button = "";
            for button in &settings.button {
                let initial_len = buttons.len();
                if initial_len == 2 {
                    break;
                }

                // Make sure buttons wont repeat
                if initial_len > 0 {
                    if first_button == button {
                        continue;
                    }
                }

                match button.as_str() {
                    "yt" => {
                        buttons.push(activity::Button::new(
                            "Search this song on YouTube",
                            &yt_url,
                        ));
                    }
                    "lastfm" => {
                        if lastfm_name.len() > 0 {
                            buttons.push(activity::Button::new("Last.fm profile", &lastfm_url));
                        }
                    }
                    "listenbrainz" => {
                        if listenbrainz_name.len() > 0 {
                            buttons.push(activity::Button::new(
                                "Listenbrainz profile",
                                &listenbrainz_url,
                            ));
                        }
                    }
                    "mprisUrl" => {
                        if mpris_url.is_empty() {
                            // if mpris url is empty or not set convert button to yt button
                            buttons.push(activity::Button::new(
                                "Search this song on YouTube",
                                &yt_url,
                            ));
                        } else {
                            if is_video_player {
                                buttons.push(activity::Button::new("Watch Now", &mpris_url));
                            } else {
                                buttons.push(activity::Button::new("Play Now", &mpris_url));
                            }
                        }
                    }
                    "shamelessAd" => {
                        buttons.push(activity::Button::new(
                            "Get This RPC",
                            "https://github.com/patryk-ku/mpris-discord-rpc",
                        ));
                    }
                    _ => continue,
                }

                // Make sure buttons wont repeat
                if initial_len < buttons.len() {
                    first_button = button;
                }
            }

            payload = match buttons.is_empty() {
                false => payload.buttons(buttons),
                true => payload,
            };

            match client.set_activity(payload) {
                Ok(_) => {
                    is_interrupted = false;
                    is_activity_set = true;
                    println!("=> Set activity [{status_text}]: {song_name}");
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
