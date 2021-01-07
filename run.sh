#!/bin/sh

set -eu

VERBOSE=false
SIMULATE=false
ALL_ARGS=$@
while [ $# -gt 0 ]; do
  key="$1"

  case $key in
    --simulate|-s)
    SIMULATE=true
    shift
    ;;
    --verbose)
    VERBOSE=true
    shift
    ;;
    *)
    shift
    ;;
  esac
done

if $VERBOSE; then
  echo `date` "Running speed test with `basename "$0"` $ALL_ARGS"
fi

if ! hash jq 2>/dev/null; then
  echo 2>&1 "No 'jq' available."
  exit 1
fi
if ! $SIMULATE && ! hash speedtest 2>/dev/null; then
  echo 2>&1 "No 'speedtest' available, install with 'apt install speedtest'."
  exit 1
fi
a="/$0"; a=${a%/*}; a=${a#/}; a=${a:-.}; DIR=$(cd "$a"; pwd)
DATA_DIR=$DIR/data
mkdir -p $DATA_DIR
NOW=`date -u`
FILE=`date -d "$NOW" +%Y%m%d%H%M%S`
CSV=$DATA_DIR/speed.csv
JSON=$DATA_DIR/$FILE.json
HEADER='date,ping,speeds_download,speeds_upload,client_ip,client_isp,server_host,server_lat,server_lon,server_location,server_country,location_distance,server_ping,server_id'
if $SIMULATE; then
  echo $HEADER
elif ! [ -f $CSV ]; then
  echo "Creating csv file $CSV."
  echo $HEADER > $CSV
fi
if $VERBOSE; then
  echo "Running speed test."
fi
if $SIMULATE; then
  RESULT='{"type":"result","timestamp":"2021-01-03T12:10:00Z","ping":{"jitter":0.28499999999999998,"latency":5.7279999999999998},"download":{"bandwidth":20309419,"bytes":176063552,"elapsed":8815},"upload":{"bandwidth":13206885,"bytes":195610380,"elapsed":15015},"packetLoss":0,"isp":"Some ISP","interface":{"internalIp":"192.168.1.2","name":"eth0","macAddr":"99:99:99:99:99:99","isVpn":false,"externalIp":"84.6.0.1"},"server":{"id":99999,"name":"Some Server","location":"São Paulo","country":"Brazil","host":"someserver.nonexistentxyz.com","port":10000,"ip":"15.22.77.1"},"result":{"id":"babad438-ac4b-47db-bc28-2de7e257bd28","url":"https://www.fakespeedtest.net/result/c/babad438-ac4b-47db-bc28-2de7e257bd28"}}'
else
  RESULT=`speedtest --accept-license --accept-gdpr --format=json --progress=no`
fi
echo $RESULT > $JSON
if $VERBOSE; then
  echo "Got result:"
  if [ -t 1 ]; then
    echo "$RESULT" | jq
  else
    echo "$RESULT"
  fi
fi
DT=`date -d "$NOW" '+%Y/%m/%d %H:%M:%S'`
if $VERBOSE; then
  echo "Writing to CSV: $CSV"
fi
RESULT=`echo $RESULT \
| jq '.ping.latency,(.download.bandwidth*8/1024/1024*100|round/100),(.upload.bandwidth*8/1024/1024*100|round/100),.interface.externalIp,.isp,.server.host,null,null,.server.location,.server.country,null,null,.server.id' \
| { printf "$DT",; paste -d, - - - - - - - - - - - - - ; }`
if $SIMULATE || $VERBOSE; then
  echo $RESULT
fi
echo $RESULT >> $CSV
if $VERBOSE; then
  echo "Done."
fi
