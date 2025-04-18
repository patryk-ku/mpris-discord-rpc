## Changes:

- Added the ability to provide your own Last.fm API key via arguments and config file.
- Providing a Last.fm API key during compilation is now optional.
- Added Listenbrainz as a fallback source for album art when Last.fm doesn't provide one or API key is not set. This can be disabled with an argument or in the config.
- Added `--xdg` flag to `enable` and `disable` subcommands that creates or removes a .desktop file for XDG Autostart as an alternative to systemd for distributions without it.
