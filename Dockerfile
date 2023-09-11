FROM rust:latest as builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM ubuntu:22.04

RUN apt update && apt install -y ca-certificates pkg-config openssl libssl-dev

WORKDIR /app

RUN useradd -r -u 1000 clineup_user
USER clineup_user

COPY --from=builder /app/target/release/clineup .

CMD ["./clineup"]
