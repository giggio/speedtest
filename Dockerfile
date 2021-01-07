FROM ubuntu:20.04
LABEL maintainer="giggio@giggio.net"
RUN apt-get update && \
    apt-get install -y gnupg2 ca-certificates && \
    apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 379CE192D401AB61 && \
    echo "deb https://ookla.bintray.com/debian generic main" | tee  /etc/apt/sources.list.d/speedtest.list && \
    apt-get update && \
    apt-get install speedtest -y
RUN apt-get install -y jq
WORKDIR /app
COPY run.sh .
ENTRYPOINT [ "/app/run.sh" ]
