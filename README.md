# mpris-discord-rpc

![img-min-optimized](https://github.com/user-attachments/assets/15990d23-51af-4d98-ae7d-7feabe84c351)

MPRIS Discord music rich presence status **with support for album covers and progress bar**. In addition, there is also an option to enable buttons with links to your profile on last.fm and a search song on Youtube. Written in Rust so it's really fast and efficient.

> [!WARNING]
> This is my first ever code written in Rust after a few days of learning so it may have bugs or errors. And I am aware that it can probably be written better but at least it works and I have been using it for long time without any problems.

## Supported players

Any player or app with [MPRIS](https://wiki.archlinux.org/title/MPRIS) support. Basically nearly every music application on Linux supports MRPIS2 in some way so there are plenty of compatible players. Web browsers also support MPRIS so this will work even with Spotify playing in Google Chrome or Firefox.

## Requirements

Any fairly new 64-bit Linux distribution. It will probably also work on older versions of Linux but would have to be manually compiled on an older system.

## Installation

Download the latest executable from the [Releases](https://github.com/patryk-ku/mpris-discord-rpc/releases) page.

Now manually start the program in terminal or add it to the system autostart.

## Configuration and usage

For the basic default usage just run executable in terminal:

```
./mpris-discord-rpc
```

> [!TIP]
> Friendly reminder to make the file executable if it doesn't work: `chmod +x mpris-discord-rpc`.

You can change the default settings using arguments. Launch executable with `-h` or `--help` for aditional info:

```
mpris-discord-rpc --help
Usage: mpris-discord-rpc [OPTIONS]

Options:
  -i, --interval <seconds>           Activity refresh rate (min 5) [default: 99999]
  -p, --profile-button <nickname>    Display "Open user's last.fm profile" button
  -y, --yt-button                    Display "Search this song on YouTube" button
  -d, --disable-cache                Disable cache (not recommended)
  -l, --list-players                 Displays all available music player names and exits. Use to get your player name for -a argument
  -a, --allowlist-add <Player Name>  Get status only from given player. Use multiple times to add several players
  -b, --debug-log                    Enable debug log
      --reset-config                 Reset config file (overwrites the old file if exists)
  -h, --help                         Print help
  -V, --version                      Print version
```

**For the best experience, I recommend using app this way:**

```
mpris-discord-rpc -i 5 -p lastfmusername -y -a 'Player Name 1' -a 'Player Name 2 etc...'
```

> [!IMPORTANT]
> After Discord recent profile layout update, users cannot see their activity buttons anymore, BUT other users can see them. This is not a bug but a feature from Discord. You can make sure the buttons work by logging into an alternative account in your browser, or just by asking a friend :)

### Allowlist

To select the music players, use the `-a`,`--allowlist-add` argument. This argument can be used multiple times to add more players. The order matters and the first is the most important.

```sh
mpris-discord-rpc -a "VLC Media Player" -a "Chrome" -a "Any other player"
```

Use the `-l`, `--list-players` to get your player name.

### Flatpak Discord fix

As flatpak applications are sandboxed this makes it difficult for any other programs to communicate with them. But this can be easily fixed using the following command:

```sh
ln -sf {app/com.discordapp.Discord,$XDG_RUNTIME_DIR}/discord-ipc-0
```

**Unfortunately but it will need to be used every reboot**. So I would also recommend adding this command to the autostart.

## System usage

As it is a very simple program and is written in the fast and efficient Rust programming language, its impact on computer performance is unnoticeable.

Normaly it uses around **11.8 MiB** of RAM but even less than **6.5 MiB** when fetching album covers only from cache.

If not disabled, the program stores the cache in `$XDG_CACHE_HOME/mpris-discord-rpc/` or `$HOME/.cache/mpris-discord-rpc/`. Don't worry, the app does not cache image files, but only the url to the image file on the last.fm server, so the cache will not take up much space.

## Compile from source

1. Install Rust and Cargo using instructions from [Rust site](https://www.rust-lang.org/).
2. Clone the repository
   ```sh
   git clone 'https://github.com/patryk-ku/mpris-discord-rpc'
   cd mpris-discord-rpc
   ```
3. Rename `.env.example` to `.env` and insert here your last.fm API key. You can easily get it [here](https://www.last.fm/pl/api).
   ```sh
   mv .env.example .env
   echo LASTFM_API_KEY=insert-key-here > .env
   ```
4. Compile executable using Cargo
   ```sh
   cargo build --release
   ```
5. The compiled executable file location is `target/release/mpris-discord-rpc`.

## Changelog

[CHANGELOG.md](CHANGELOG.md)
