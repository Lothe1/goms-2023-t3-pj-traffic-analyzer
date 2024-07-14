#!/bin/bash
influx config rm default
# Setup InfluxDB
influx setup \
  --username admin \
  --password password \
  --token ball \
  --org doglver \
  --bucket db \
  --retention 24h\
  --force