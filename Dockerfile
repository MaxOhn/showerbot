FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates gosu && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN mkdir -p /app/maps

COPY showerbot entrypoint.sh ./
RUN chmod +x entrypoint.sh

VOLUME ["/app/maps"]
ENV MAP_PATH=/app/maps

ENTRYPOINT ["./entrypoint.sh"]
