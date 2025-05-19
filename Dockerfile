FROM rust:1.87 AS builder
WORKDIR /usr/src/javelot
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/javelot /usr/local/bin/javelot
CMD ["javelot"]
