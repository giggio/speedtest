FROM alpine:3 as base
LABEL maintainer="giggio@giggio.net"
ENTRYPOINT [ "/usr/bin/trackspeedtest" ]
RUN wget https://bintray.com/ookla/download/download_file?file_path=ookla-speedtest-1.0.0-x86_64-linux.tgz -O speedtest.tgz && \
    tar -xvzf speedtest.tgz && \
    mv ./speedtest /usr/bin/ && \
    rm speedtest.*

FROM rust:1.49-alpine as build
RUN apk add --no-cache musl-dev
RUN mkdir ./src/ && echo "fn main() {}" > ./src/dummy.rs
COPY Cargo.toml Cargo.lock ./
RUN sed -i '/# package/ i [[bin]] \nname = "trackspeedtest" \npath = "src/dummy.rs"' Cargo.toml && cargo build --release
COPY . .
RUN rm ./src/dummy.rs && cargo build --release
RUN strip target/release/trackspeedtest

FROM base
COPY --from=build target/release/trackspeedtest /usr/bin/