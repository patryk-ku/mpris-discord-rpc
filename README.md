# mpris-discord-rpc
MPRIS2 Discord music rich presence status **with support for album covers**. In addition, there is also an option to enable buttons with links to your profile on last.fm and a search song on Youtube. Written in Rust so it's really fast and efficient.

![rpc](https://github.com/patryk-ku/mpris-discord-rpc/assets/38609910/808b88cf-243a-4ec1-a5d4-9669f396e9b0)

> **⚠️ Warning:** This is my first ever code written in Rust after a few days of learning so it may have bugs or errors. And I am aware that it can probably be written better but at least it works and I have been using it for some time without any problems.

## Supported players

Any player or app with [MPRIS2](https://wiki.archlinux.org/title/MPRIS) support. Basically nearly every music application on Linux supports MRPIS2 in some way so there are plenty of compatible players.

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

You can change the default settings using arguments. Launch executable with `-h` or `--help` for aditional info:

```
./mpris-discord-rpc --help
Usage: mpris-discord-rpc [OPTIONS]

Options:
  -i, --interval <seconds>         Activity refresh rate (min 5) [default: 10]
  -p, --profile-button <nickname>  Display "Open user's last.fm profile" button
  -y, --yt-button                  Display "Search this song on YouTube" button
  -d, --disable-cache              Disable cache (not recommended)
  -h, --help                       Print help
  -V, --version                    Print version
```

**For the best experience, I recommend using app this way:**

```
./mpris-discord-rpc -i 5 -p lastfmusername -y
```

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
5. Compile executable using Cargo
   ```sh
   cargo build --release
   ```
6. The compiled executable file location is `target/release/mpris-discord-rpc`.
