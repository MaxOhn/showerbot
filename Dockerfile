FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY showerbot .

VOLUME ["/maps"]
ENV MAP_PATH=/maps

ENTRYPOINT ["./showerbot"]
