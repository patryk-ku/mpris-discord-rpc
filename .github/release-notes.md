## Changes:

- If no album art is found on Last.fm, use the `artUrl` provided by MPRIS if it exists. This is especially useful for movies played in a browser, e.g., YouTube. If a YouTube thumbnail URL is detected, replace the player icon with the YouTube icon. This can be disabled with an argument or in the config.
- Added the option to mark players as video players, which will display the status "Watching Video" and make the RPC more appropriate for movies. A video thumbnail will be displayed if available as `artUrl` in MPRIS.
- Added a new `mprisUrl` button that can link to the currently playing content if MPRIS provides such information.
- The systemd unit file is now installed by the package manager instead of manually by the program.
- Added the ability to force a different player icon and name than is actually used.
