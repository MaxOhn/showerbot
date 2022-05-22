# Step 1:
# Gather dependencies to avoid the bug
# https://github.com/docker/buildx/issues/395
FROM --platform=$BUILDPLATFORM rust:slim-bullseye as builder-source

WORKDIR /build
COPY . .
RUN mkdir .cargo && cargo vendor > .cargo/config

# Step 2:
# Build bot on the target platform
FROM rust:slim-bullseye as builder

WORKDIR /build
COPY --from=builder-source /build .
RUN cargo build --release --offline

# Step 3:
# Move binary into smaller environment
FROM debian:bullseye-slim

RUN apt update && apt upgrade -y && apt install ca-certificates -y
COPY --from=builder /build/target/release/showerbot /usr/local/bin/showerbot

RUN touch .env && mkdir -p /usr/local/beatmaps/
WORKDIR /usr/local/showerbot/

CMD ["showerbot"]