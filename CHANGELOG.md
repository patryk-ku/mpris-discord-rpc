## v0.2.2

- Fixed: Incorrect album art display when the album artist differed from the track artist.
- Updated dependencies to latest versions.

## v0.2.1

- Added option to hide album title in activity.
- Added the ability to customize displayed icon adjacent to album artwork. Available options: none, play/pause icons, music player icon, Last.fm avatar.

## v0.2.0

- (⚠️ **Breaking**) The `-n`, `--player-name` argument has been removed. Use `-a`, `--allowlist-add` instead.
- (⚠️ **Breaking**) The previous button arguments (`-p`, `--profile-button`, `-y`, `--yt-button`) have been consolidated into a single new argument: `-b, --button` with options: `yt`, `lastfm`, `listenbrainz` and separate arguments for setting service usernames. More additional buttons coming in the future.
- Added support for configuring the program via a configuration file.
- Added commands for easy setup of autostart using systemd.
- Now available as `.deb` and `.rpm` packages, and in the AUR.
- Print debug logs with `--debug-log`.

## v0.1.5

- Set Discord RPC activity type to "Listening".
- Listening progress bar similar to Spofity.
- Album name is now displayed.
- Fixed detection of track duration, current position and start/end time calculation.
- From now on, the program checks if the directory to which it tries to save the cache file exists. The cache should now work properly.

## v0.1.4

- Allowlist of music players (`-a` or `--allowlist-add`).

## v0.1.3

- Dependencies update.

## v0.1.2

- Dependencies update.

## v0.1.1

- List active MPRIS2 players with `-l` or `--list-players` arguments.
- Select only one specific player for music status with `-n` or `--player-name` arguments.
- Fix: skip setting status when unknown metadata.
- Better log messages.
- Switched to different cache library.

## v0.1.0

- Initial release.
