FROM rust:1.81.0-alpine AS builder

RUN apk add alpine-sdk openssl openssl-dev perl

RUN mkdir -p /build
WORKDIR /build
COPY Cargo.toml /build/
RUN mkdir -p /build/src
RUN echo "fn main() {}" > /build/src/main.rs
RUN cargo build --release
COPY src /build/src
RUN touch /build/src/main.rs
RUN cargo build --release

FROM scratch
COPY --from=builder /build/target/release/container-cast /
COPY etc_passwd /etc/passwd
USER 65534
ENTRYPOINT ["/container-cast"]
