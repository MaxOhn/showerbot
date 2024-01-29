# Showerbot

Discord bot for [osu!](https://osu.ppy.sh) based on [Bathbot](https://github.com/MaxOhn/Bathbot).

This is a self-host bot meaning you can't invite an existing bot, you must run it yourself.

## Commands

It only has the following commands:
- Prefix:
    - `<help`: Display help for prefix commands
    - `<ping`: Check if the bot is online
    - `<nlb`: Display the national leaderboard of a map
- Slash:
  - `/pingnlb`
  - `/nlb`


## Setup

- Before starting, be sure you have access to an [osu!](https://osu.ppy.sh/home) account that has supporter. The bot will be able to show the national map leaderboards of that user's country.
- Download a binary from the [releases](https://github.com/MaxOhn/showerbot/releases) page for your operating system.
- Copy-paste the content of the [`.env.example`](https://github.com/MaxOhn/showerbot/blob/main/.env.example) file into a file called `.env` and put it next to the downloaded binary.
- Read through the `.env` file and fill in all required variables
- Run the binary