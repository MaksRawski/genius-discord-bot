FROM rust

WORKDIR /genius
COPY . .
RUN apt-get update && apt-get install -y -f jq libjq-dev libonig5 libonig-dev openssl ca-certificates fonts-lato imagemagick

RUN JQ_LIB_DIR=/usr/lib/x86_64-linux-gnu/libjq.so.1 cargo build --release

CMD LANG=en_US.UTF-8 LANGUAGE=en.UTF-8 cargo run --release
