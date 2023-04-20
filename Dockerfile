FROM clux/muslrust:latest AS build
WORKDIR /usr/src

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/coubot
COPY . .
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM alpine:latest
RUN apk add --no-cache ffmpeg
COPY --from=build /root/.cargo/bin/coubot /usr/local/bin/coubot
CMD ["coubot"]
