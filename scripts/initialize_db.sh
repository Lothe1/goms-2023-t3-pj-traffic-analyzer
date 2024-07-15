#!/bin/bash

wait_for_influxdb() {
    until curl -s -I -XGET http://influxdb:8086/ping > /dev/null; do
        echo "InfluxDB is not yet ready - waiting..."
        sleep 2
    done
}


wait_for_influxdb

influx config rm default
# Setup InfluxDB
influx setup \
  --username admin \
  --password password \
  --token ball \
  --org doglver \
  --bucket db \
  --retention 24h\
  --force \
  --host http://influxdb:8086 