FROM rust

WORKDIR /genius
COPY . .
RUN apt-get update && apt-get install -y -f fonts-lato imagemagick openssl ca-certificates

ENV JQ_LIB_DIR=/usr/lib/libjq.so
RUN cargo build --release

# there is probably a better way of going about this
CMD while :; do /genius/target/release/genius; sleep 10; done
