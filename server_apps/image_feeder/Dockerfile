FROM rust:1.92 as builder

ENV SQLX_OFFLINE=true

RUN apt-get update
RUN apt-get install cmake -y

WORKDIR /usr/src/image_feeder
# Local dependency
RUN mkdir /usr/src/db_storage 
COPY db_storage/. /usr/src/db_storage/
# Service code
COPY image_feeder/. .
RUN cp -a ./.sqlx ../db_storage/.sqlx

RUN cargo install --path .

# FROM scratch
FROM debian:trixie-slim

COPY --from=builder /usr/local/cargo/bin/image_feeder /usr/local/bin/image_feeder
CMD ["image_feeder"]

