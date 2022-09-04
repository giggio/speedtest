FROM alpine:3.16 as bins
ARG PLATFORM
RUN wget https://install.speedtest.net/app/cli/ookla-speedtest-1.2.0-linux-${PLATFORM}.tgz -O speedtest.tgz && \
    tar -xvzf speedtest.tgz && \
    mv ./speedtest /usr/bin/ && \
    rm speedtest.*
COPY target/output/trackspeedtest /app/trackspeedtest
RUN apk add binutils && strip /app/trackspeedtest

FROM opensuse/leap:15.4 as opensuse
RUN ldd /bin/echo | tr -s '[:blank:]' '\n' | grep '^/' | \
    xargs -I % sh -c 'mkdir -p $(dirname deps%); cp % deps%;'

FROM scratch
LABEL maintainer="giggio@giggio.net"
ENTRYPOINT [ "/trackspeedtest" ]
COPY --from=bins /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=bins /usr/bin/speedtest .
COPY --from=opensuse /bin/echo .
COPY --from=opensuse  /deps /
COPY --from=bins /app/trackspeedtest .
