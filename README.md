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
- Compose docker images: `docker-compose -f {file} up -d` with `{file}` being:
  - `docker-compose.amd.yml` if you run the bot on windows or a linux amd based platform (ubuntu, ...)
  - `docker-compose.arm.yml` if you run the bot on an arm based system like raspberry or macOS with an M1

If there is an error along the lines of

```
thread 'main' panicked at 'called Result::unwrap() on an Err value: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }'
```

you can un-comment the line `privileged: true` in the corresponding `docker-compose.{amd/arm}.yml` file and retry.

While running:

- You can find the logs in `/docker-volume/logs`
- To see live console output, run `docker attach showerbot`
- To shut down, run `docker-compose -f {file} down`

## Manual setup

[rust](https://www.rust-lang.org/) and [postgres](https://www.postgresql.org/) must be installed.

- Clone the repo: `git clone https://github.com/MaxOhn/showerbot`
- Install [sqlx](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli) via `cargo install sqlx-cli --no-default-features --features postgres,rustls`
- Rename `.env.example` to `.env` and edit the file as described inside
- Run `sqlx database create && sqlx migrate run` to prepare the database
- Compile and run: `cargo run --release`
