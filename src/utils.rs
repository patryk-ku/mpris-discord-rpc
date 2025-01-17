use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use std::fs;
use std::path::PathBuf;
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

pub fn enable_service(home_dir: &PathBuf) {
    let service_dir = home_dir.join(".config/systemd/user");
    let service_file = service_dir.join("mpris-discord-rpc.service");

    let service_text = r#"[Unit]
Description=MPRIS Discord music rich presence status with support for album covers and progress bar
After=network.target

[Service]
ExecStart=/usr/bin/mpris-discord-rpc
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
"#;

    match fs::create_dir_all(&service_dir) {
        Err(_) => {
            println!("Failed to create user systemd services directory.");
            process::exit(1);
        }
        Ok(_) => match fs::write(&service_file, service_text) {
            Ok(_) => println!("Created systemd service file: {}", service_file.display()),
            Err(_) => {
                println!("Failed to create user systemd service file.");
                process::exit(1);
            }
        },
    }

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
