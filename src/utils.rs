use std::fs;
use std::path::PathBuf;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient};

// Use to print debug log if enabled with argument
#[macro_export]
macro_rules! debug_log {
    ($cond:expr, $($arg:tt)*) => {
        if $cond {
            println!("\x1b[34;1m[debug]\x1b[0m {}", format!($($arg)*));
        }
    };
}

pub fn create_config_file(home_dir: &PathBuf, force: bool) -> PathBuf {
    let config_dir = home_dir.join(".config/mpris-discord-rpc");
    let config_file = config_dir.join("config.toml");

    if config_file.exists() && !force {
        return config_file;
    }

    let config_text = r#" # mpris-discord-rpc config file
test = true
"#;

    match fs::create_dir_all(&config_dir) {
        Err(_) => println!("[config] Failed to create config directory."),
        Ok(_) => match fs::write(&config_file, config_text) {
            Ok(_) => println!(
                "[config] Created new config file: {}.",
                config_file.display()
            ),
            Err(_) => println!("[config] Error: Failed to create config file."),
        },
    }

    return config_file;
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
