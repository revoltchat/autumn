# Build Stage
FROM ekidd/rust-musl-builder:nightly-2021-01-01 AS builder
WORKDIR /home/rust/src

RUN USER=root cargo new --bin autumn
WORKDIR /home/rust/src/autumn
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Bundle Stage
FROM alpine:latest
RUN apk update && apk add ca-certificates && rm -rf /var/cache/apk/*
COPY --from=builder /home/rust/src/autumn/target/x86_64-unknown-linux-musl/release/autumn ./
# ! FIXME: bundle static ffprobe instead of everything
RUN apk add --no-cache ffmpeg
EXPOSE 3000
ENV AUTUMN_HOST 0.0.0.0:3000
CMD ["./autumn"]
