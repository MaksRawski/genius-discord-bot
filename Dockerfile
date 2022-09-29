FROM rust

WORKDIR /genius
COPY . .

RUN ls -a

RUN apt-get update
RUN apt-get install -y -f jq libjq-dev libonig5 libonig-dev openssl ca-certificates fonts-lato imagemagick

RUN ls -a
RUN ls -a $CARGO_HOME/

# jq's lib sometimes isn't found
RUN JQ_LIB_DIR=/usr/lib/x86_64-linux-gnu/libjq.so.1 cargo build --release

CMD cargo run --release
