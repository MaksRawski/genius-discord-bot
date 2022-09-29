FROM rust

WORKDIR /genius
COPY . .

RUN apt-get update
RUN apt-get install -y -f jq libjq-dev libonig5 libonig-dev openssl ca-certificates fonts-lato imagemagick

RUN ls /usr/local/cargo -la
# jq's lib sometimes isn't found
ENV JQ_LIB_DIR=/usr/lib/x86_64-linux-gnu/libjq.so.1
RUN cargo build --release

CMD cargo run --release
