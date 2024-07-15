FROM influxdb:latest


COPY scripts/initialize_db.sh /scripts/initialize_db.sh
RUN chmod +x /scripts/initialize_db.sh
CMD ["/initialize_db.sh"]
