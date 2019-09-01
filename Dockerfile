FROM clux/muslrust:latest AS build
WORKDIR /usr/src

RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new coubot
WORKDIR /usr/src/coubot
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM alpine:latest
RUN apk add --no-cache ffmpeg
COPY --from=build /root/.cargo/bin/coubot .
CMD ["./coubot"]
