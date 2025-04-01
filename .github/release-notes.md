## Changes:

- Added the ability to force a different player icon and name than is actually used.
- If no album art is found on Last.fm, use the `artUrl` provided by MPRIS if it exists. This is especially useful for movies played in a browser, e.g., YouTube. This can be disabled with an argument or in the config.
- If a YouTube thumbnail URL is detected, replace the player icon with the YouTube icon. This can be disabled with an argument or in the config.
