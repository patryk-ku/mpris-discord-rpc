## Changes:

- **BREAKING CHANGE:** The `-n`, `--player-name` argument has been removed. Use `-a`, `--allowlist-add` instead.
- **BREAKING CHANGE:** The previous button arguments (`-p`, `--profile-button`, `-y`, `--yt-button`) have been consolidated into a single new argument: `-b, -button` with options: `yt`, `lastfm`, `listenbrainz` and separate arguments for setting service usernames. More additional buttons coming in the future.
- Added support for configuring the program via a configuration file.
- Added commands for easy setup of autostart using systemd.
- Now available as `.deb` and `.rpm` packages, and in the AUR.
- Print debug logs with `--debug-log`.
