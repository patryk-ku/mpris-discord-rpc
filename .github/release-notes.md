> [!IMPORTANT]
> The project has been renamed to **music-discord-rpc**, and updating requires manual intervention.

### Quick instructions:

- First, stop the daemon and disable its autostart:

```sh
# Systemd distributions (Ubuntu, Fedora, Arch, etc)
mpris-discord-rpc disable

# XDG Autostart for distrubutions without systemd (Void and others)
mpris-discord-rpc disable --xdg
pkill -f mpris-discord-rpc
```

- Next, uninstall the package using the appropriate command for your distribution.
- Move your old configuration file. You can also do this later. In that case, you won't need to create the directory, but the service will require a restart to load the config.

```sh
# Instructions if you keep your configuration files in the default location
mkdir -p ~/.config/music-discord-rpc
mv ~/.config/mpris-discord-rpc/config.yaml ~/.config/music-discord-rpc/config.yaml
```

- Install the new package following the [instructions](https://github.com/patryk-ku/music-discord-rpc?tab=readme-ov-file#installation) in the README.md.
- Start the program as a background daemon and enable autostart:

```sh
# Systemd distributions (Ubuntu, Fedora, Arch, etc)
music-discord-rpc enable

# XDG Autostart for distrubutions without systemd (Void and others).
# Only adds to autostart (service will start after a system restart)."
music-discord-rpc enable --xdg
```

### Why the name change?

I renamed the program because I ported it to macOS (and soon to Windows), and the old name with "mpris" no longer fit. Sorry for any inconvenience this may cause.

## Changes:

- Ported to MacOS.
- Renamed project from **mpris-discord-rpc** to **music-discord-rpc**.
- Added `--get-player-id` flag for easier retrieval of Player ID when requesting missing icons.
