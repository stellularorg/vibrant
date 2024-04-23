FROM ubuntu:noble

RUN apt-get update && apt-get install -y git bash curl
COPY ./init_container.sh /init.sh
CMD ["/init.sh"]
