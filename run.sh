#!/bin/bash

# Set up kafka docker
docker run --name kafka -d -p 9092:9092 apache/kafka:3.7.1

docker exec -it kafka /opt/kafka/bin/kafka-topics.sh --create --topic listener-to-enricher --bootstrap-server localhost:9092

docker exec -it kafka /opt/kafka/bin/kafka-topics.sh --create --topic enricher-to-tsdb --bootstrap-server localhost:9092

# Set up Influxdb (TSDB)
docker run --name influxdb -d -p 8086:8086 \
  -v "$PWD/data:/var/lib/influxdb2" \
  -v "$PWD/config:/etc/influxdb2" \
  -e DOCKER_INFLUXDB_INIT_MODE=setup \
  -e DOCKER_INFLUXDB_INIT_USERNAME=ROOTNAME \
  -e DOCKER_INFLUXDB_INIT_PASSWORD=CHANGEME123 \
  -e DOCKER_INFLUXDB_INIT_ORG=doglover645 \
  -e DOCKER_INFLUXDB_INIT_BUCKET=traffic-analyzer \
  influxdb:2

# Get token and pass it to at the start of the applicatoin by commandline probably 
influx auth create \
  --org doglover645 \
  --all-access

