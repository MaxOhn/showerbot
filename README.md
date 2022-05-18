# Showerbot

Discord bot for [osu!](https://osu.ppy.sh) based on [Bathbot](https://github.com/MaxOhn/Bathbot).

This is a self-host bot meaning you can't invite an existing bot, you must run it yourself.

## Commands

It only has the following commands:
- Prefix:
    - `<help`: Display help for prefix commands
    - `<ping`: Check if the bot is online
    - `<prefix`: Change the prefix for a server
    - `<nlb`: Display the national leaderboard of a map
- Slash:
  - `/ping`
  - `/nlb`

## Requirements

- You must have access to an [osu!](https://osu.ppy.sh/home) account that has supporter. The bot will be able to show the national map leaderboards of that user's country
- [git](https://git-scm.com/) must be installed

## Setup through docker (recommended)

[docker](https://www.docker.com/) must be installed.

- Clone the repo: `git clone https://github.com/MaxOhn/showerbot`
- Rename `.env.example` to `.docker.env` and edit the file as described inside
- Compose docker images: `docker-compose up -d`

## Manual setup

[rust](https://www.rust-lang.org/) and [postgres](https://www.postgresql.org/) must be installed.

- Clone the repo: `git clone https://github.com/MaxOhn/showerbot`
- Install [sqlx](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli) via `cargo install sqlx-cli --no-default-features --features postgres,rustls`
- Rename `.env.example` to `.env` and edit the file as described inside
- Run `sqlx database create && sqlx migrate run` to prepare the database
- Compile and run: `cargo run --release`

## While running

- You can find the logs in `/docker-volume/logs`
- To see live console output, run `docker attach showerbot`
- To shut down, run `docker-compose down`