# Step 1:
# Gather dependencies to avoid the bug
# https://github.com/docker/buildx/issues/395
FROM --platform=$BUILDPLATFORM rust:slim-bullseye as builder-source

RUN apt update && apt install -y git
RUN git clone https://github.com/launchbadge/sqlx /usr/src/sqlx
WORKDIR /usr/src/sqlx
RUN mkdir .cargo && cargo vendor > .cargo/config

WORKDIR /usr/src/showerbot
COPY . .
RUN mkdir .cargo && cargo vendor > .cargo/config

# Step 2:
# Build sqlx and the bot on the target platform
FROM rust:slim-bullseye as builder

WORKDIR /usr/src/sqlx
COPY --from=builder-source /usr/src/sqlx .
RUN cargo install --path /usr/src/sqlx/sqlx-cli --no-default-features --features postgres,rustls --offline

WORKDIR /usr/src/showerbot
COPY --from=builder-source /usr/src/showerbot .
RUN cargo build --release --offline

# Step 3:
# Move binaries into smaller environment
FROM debian:bullseye-slim

COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

RUN apt update && apt upgrade -y && apt install ca-certificates -y
COPY --from=builder /usr/src/showerbot/target/release/showerbot /usr/local/bin/showerbot

RUN touch .env && mkdir -p /usr/local/beatmaps/
WORKDIR /usr/local/showerbot/
COPY ./migrations ./migrations
COPY ./start.sh .
RUN chmod +x ./start.sh
WORKDIR /var/local/showerbot

CMD ["sh", "/usr/local/showerbot/start.sh"]