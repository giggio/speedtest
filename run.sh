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
NOW=`date -u`
FILE=`date -d "$NOW" +%Y%m%d%H%M%S`
CSV=$DATA_DIR/speed.csv
JSON=$DATA_DIR/$FILE.json
if ! [ -f $CSV ]; then
  echo "Creating cs file $CSV."
  echo 'date,ping,speeds_download,speeds_upload,client_ip,client_isp,server_host,server_lat,server_lon,server_location,server_country,location_distance,server_ping,server_id' > $CSV
fi
echo "Running speed test $CSV."
RESULT=`speed-test -vj`
#RESULT='{"ping":14,"download":121,"upload":62.3,"data":{"speeds":{"download":121.039,"upload":62.33,"originalDownload":13330241,"originalUpload":6840412},"client":{"ip":"0.0.0.0","lat":-3.038,"lon":-51.333,"isp":"Comcast","isprating":2.4,"rating":0,"ispdlavg":0,"ispulavg":0,"country":"BR"},"server":{"host":"speedtest.foobar.com.br:1234","lat":2.11,"lon":6.11,"location":"NYC","country":"Brazil","cc":"BR","sponsor":"Telecom America","distance":37.7,"distanceMi":1.6,"ping":13.9,"id":"99999"}}}'
echo $RESULT > $JSON
echo "Got result:"
if [ -t 1 ]; then
  echo "$RESULT" | jq
else
  echo "$RESULT"
fi
DT=`date -d "$NOW" '+%Y/%m/%d %H:%M:%S'`
echo "Writing to CSV: $CSV"
jq .ping,.data.speeds.download,.data.speeds.upload,.data.client.ip,.data.client.isp,.data.server.host,.data.server.lat,.data.server.lon,.data.server.location,.data.server.country,.data.location.distance,.data.server.ping,.data.server.id $JSON \
| { printf "$DT",; paste -d, - - - - - - - - - - - - - ; } >> $CSV
echo "Done."
