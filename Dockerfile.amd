FROM rust:slim-buster as builder

WORKDIR /build
COPY . .

RUN cargo build --release

FROM debian:buster-slim

RUN apt update && apt upgrade && apt install ca-certificates -y
COPY --from=builder /build/target/release/showerbot /usr/local/bin/showerbot

RUN touch .env && mkdir -p /usr/local/beatmaps/
WORKDIR /usr/local/showerbot/

CMD ["showerbot"]