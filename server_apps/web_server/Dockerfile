FROM rust:1.92 as builder

ENV SQLX_OFFLINE=true

RUN apt-get update 
RUN apt-get install protobuf-compiler -y

WORKDIR /usr/src/web_server
# Local dependency
COPY db_storage/. ../db_storage/
# Service code 
COPY web_server/. .
RUN cp -a ./.sqlx ../db_storage/.sqlx

RUN cargo install --path .
# ---
FROM debian:trixie-slim

RUN apt-get update 
RUN apt-get install libsodium-dev -y

EXPOSE 50051
EXPOSE 8000 

COPY --from=builder /usr/local/cargo/bin/web_server /usr/local/bin/web_server
CMD ["web_server"]

