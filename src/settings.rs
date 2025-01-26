use clap_serde_derive::{
    clap::{self, Parser, Subcommand},
    serde::Serialize,
    ClapSerde,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use crate::debug_log;

#[derive(Parser, ClapSerde, Serialize, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Activity refresh rate (min 5, default 10)
    #[arg(short, long, value_name = "seconds", value_parser = clap::value_parser!(u64).range(5..))]
    pub interval: Option<u64>,

    /// Select visible buttons
    #[arg(short, long, value_name = "name", value_parser = ["yt", "lastfm", "listenbrainz", "shamelessAd"])]
    pub button: Vec<String>,

    /// Your Last.fm nickname
    #[arg(long, value_name = "nickname", value_parser = clap::value_parser!(String))]
    pub lastfm_name: Option<String>,

    /// Your Listenbrainz nickname
    #[arg(long, value_name = "nickname", value_parser = clap::value_parser!(String))]
    pub listenbrainz_name: Option<String>,

    /// Select the icon displayed next to the album cover (default playPause)
    #[arg(short, long, value_name = "name", value_parser = ["playPause", "player", "lastfmAvatar", "none"])]
    pub small_image: Option<String>,

    /// Displays all available music player names and exits. Use to get your player name for -a argument
    #[arg(short, long)]
    #[serde(skip_deserializing)]
    pub list_players: bool,

    /// Get status only from given player. Use multiple times to add several players.
    #[arg(short = 'a', long = "allowlist-add", value_name = "Player Name", value_parser = clap::value_parser!(String))]
    pub allowlist: Vec<String>,

    /// Hide album name
    #[arg(long)]
    pub hide_album_name: bool,

    /// Disable cache (not recommended)
    #[arg(short, long)]
    pub disable_cache: bool,

    /// Show debug log
    #[arg(long)]
    #[serde(skip_deserializing)]
    pub debug_log: bool,

    /// Reset config file (overwrites the old file if exists)
    #[arg(long)]
    #[serde(skip_deserializing)]
    pub reset_config: bool,

    /// Recursive fields
    #[serde(skip_deserializing)]
    #[command(flatten)]
    pub suboptions: SubConfig,
}

#[derive(Debug, Parser, Default, Serialize)]
pub struct SubConfig {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Serialize)]
pub enum Commands {
    /// Start RPC in the background and enable autostart
    Enable {},
    /// Stop RPC and disable autostart
    Disable {},
    /// Use to restart the service and reload the changed configuration file.
    Restart {},
}

// Use to get config path, create new config or reset existing
fn create_config_file(home_dir: &PathBuf, force: bool) -> (bool, PathBuf) {
    let config_dir = home_dir.join(".config/mpris-discord-rpc");
    let config_file = config_dir.join("config.yaml");

    if config_file.exists() && !force {
        return (true, config_file);
    }

    let config_text = r#"# mpris-discord-rpc configuration file

# You can reset this file using the command:
# mpris-discord-rpc --reset-config
# Or you can manually copy the example config from repo:
# https://github.com/patryk-ku/mpris-discord-rpc/blob/main/config.yaml

# Activity refresh rate in seconds (min 5)
interval: 10

# Select visible activity buttons (max 2) [possible values: yt, lastfm, listenbrainz, shamelessAd]
# button:
#   - yt
#   - lastfm

# Uncomment and enter your nicknames for activity buttons
# lastfm_name: "nickname"
# listenbrainz_name: "nickname"

# Select the icon displayed next to the album cover (default playPause) [possible values: playPause, player, lastfmAvatar, none]
small_image: playPause

# Only use the status from the following music players
# Use -l, --list-players to get player exact name to use with this option
# The order matters and the first is the most important.
# allowlist:
#   - "VLC Media Player"
#   - "Chrome"
#   - "Any other player"

# Hide the album name to decrease activity height
hide_album_name: false

# Disable cache (not recommended)
disable_cache: false
"#;

    match fs::create_dir_all(&config_dir) {
        Err(_) => {
            println!("[config] Failed to create config directory.");
            return (false, config_file);
        }
        Ok(_) => match fs::write(&config_file, config_text) {
            Ok(_) => println!(
                "[config] Created new config file: {}",
                config_file.display()
            ),
            Err(_) => {
                println!("[config] Error: Failed to create config file.");
                return (false, config_file);
            }
        },
    }

    return (true, config_file);
}

// Used to get settings merged from args and config file
pub fn load_settings() -> Cli {
    let (home_exists, home_dir) = match env::var("HOME") {
        Ok(val) => (true, PathBuf::from(val)),
        Err(_) => (false, PathBuf::from("/")),
    };

    let args = Cli::parse();
    debug_log!(args.debug_log, "Debug logs: enabled.");
    debug_log!(args.debug_log, "args: {:#?}", args);

    // Reset config file is user used --reset-config and exit
    if args.reset_config {
        create_config_file(&home_dir, true);
        process::exit(0);
    }

    if !home_exists {
        return args;
    }

    let (mut config_exists, config_file) = create_config_file(&home_dir, false);
    if !config_exists {
        return args;
    }

    // Read user config file
    let mut config = match fs::read_to_string(&config_file) {
        Ok(yaml_str) => match serde_yaml::from_str::<<Cli as ClapSerde>::Opt>(&yaml_str) {
            Ok(yaml_args) => Cli::from(yaml_args),
            Err(error) => {
                println!("Failed to parse config file: {}", error);
                config_exists = false;
                Cli::from_clap()
            }
        },
        Err(_) => {
            println!("Failed to read config file.");
            config_exists = false;
            Cli::from_clap()
        }
    };

    if !config_exists {
        return args;
    }
    println!("Configuration loaded from file: {}", config_file.display());
    debug_log!(args.debug_log, "config: {:#?}", config);

    // Logic of merging config with args
    if args.interval != config.interval && args.interval.is_some() {
        config.interval = args.interval;
    }

    if args.button != config.button && args.button.len() > 0 {
        config.button = args.button;
    }

    if args.lastfm_name != config.lastfm_name && args.lastfm_name.is_some() {
        config.lastfm_name = args.lastfm_name;
    }

    if args.listenbrainz_name != config.listenbrainz_name && args.listenbrainz_name.is_some() {
        config.listenbrainz_name = args.listenbrainz_name;
    }

    if args.small_image != config.small_image && args.small_image.is_some() {
        config.small_image = args.small_image;
    }

    if args.hide_album_name {
        config.hide_album_name = args.hide_album_name;
    }

    if args.disable_cache {
        config.disable_cache = args.disable_cache;
    }

    if args.list_players {
        config.list_players = args.list_players;
    }

    if args.allowlist != config.allowlist && args.allowlist.len() > 0 {
        config.allowlist = args.allowlist;
    }

    if args.debug_log {
        config.debug_log = args.debug_log;
    }

    if args.reset_config {
        config.reset_config = args.reset_config;
    }

    config.suboptions = args.suboptions;

    return config;
}
