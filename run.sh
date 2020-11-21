#!/bin/sh

if ! hash jq 2>/dev/null; then
  echo 2>&1 "No 'jq' available."
  exit 1
fi
if ! hash speed-test 2>/dev/null; then
  echo 2>&1 "No 'speed-test' available, install with 'npm i -g speed-test'."
  exit 1
fi
a="/$0"; a=${a%/*}; a=${a#/}; a=${a:-.}; DIR=$(cd "$a"; pwd)
DATA_DIR=$DIR/data
mkdir -p $DATA_DIR
FILE=`date +%Y%m%d%H%M`
CSV=$DATA_DIR/speed.csv
JSON=$DATA_DIR/$FILE.json
if ! [ -f $CSV ]; then
  echo 'ping,speeds_download,speeds_upload,client_ip,client_isp,server_host,server_lat,server_lon,server_location,server_country,location_distance,server_ping,server_id' > $CSV
fi
speed-test -vj > $JSON
jq .ping,.data.speeds.download,.data.speeds.upload,.data.client.ip,.data.client.isp,.data.server.host,.data.server.lat,.data.server.lon,.data.server.location,.data.server.country,.data.location.distance,.data.server.ping,.data.server.id $JSON \
| paste -d, - - - - - - - - - - - - - >> $CSV
