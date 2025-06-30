# Dockerfile
FROM rust:1.77 as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app

COPY --from=builder /app/target/release/http-server-actix /app/
EXPOSE 8080

CMD ["./http-server-actix"]
