FROM ubuntu:20.04
LABEL maintainer="giggio@giggio.net"
WORKDIR /app
RUN apt-get update && \
    apt-get install -y gnupg2 ca-certificates && \
    apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 379CE192D401AB61 && \
    echo "deb https://ookla.bintray.com/debian generic main" > /etc/apt/sources.list.d/speedtest.list && \
    apt-get update && \
    apt-get install -y speedtest jq
COPY run.sh .
ENTRYPOINT [ "/app/run.sh" ]