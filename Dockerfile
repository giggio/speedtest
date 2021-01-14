FROM rust:1.49-alpine as speedtestbin
RUN wget https://bintray.com/ookla/download/download_file?file_path=ookla-speedtest-1.0.0-x86_64-linux.tgz -O speedtest.tgz && \
    tar -xvzf speedtest.tgz && \
    mv ./speedtest /usr/bin/ && \
    rm speedtest.*

FROM opensuse/leap:15.2 as opensuse
RUN ldd /bin/echo | tr -s '[:blank:]' '\n' | grep '^/' | \
    xargs -I % sh -c 'mkdir -p $(dirname deps%); cp % deps%;'

FROM rust:1.49-alpine as build
WORKDIR /app
RUN apk add --no-cache build-base musl-dev perl
RUN mkdir ./src/ && echo "fn main() {}" > ./src/dummy.rs
COPY Cargo.toml Cargo.lock ./
RUN sed -i '/# package/ i [[bin]] \nname = "trackspeedtest" \npath = "src/dummy.rs"' Cargo.toml && cargo build --release --target x86_64-unknown-linux-musl
COPY . .
RUN rm ./src/dummy.rs && cargo build --release --target x86_64-unknown-linux-musl --features static
RUN strip /app/target/x86_64-unknown-linux-musl/release/trackspeedtest

FROM scratch
LABEL maintainer="giggio@giggio.net"
ENTRYPOINT [ "/trackspeedtest" ]
COPY --from=speedtestbin /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=speedtestbin /usr/bin/speedtest .
COPY --from=opensuse /bin/echo .
COPY --from=opensuse  /deps /
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/trackspeedtest .
