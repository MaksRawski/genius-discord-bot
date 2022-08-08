FROM rust

WORKDIR /usr/src/genius-bot
COPY . .
RUN apt-get update -y
RUN apt-get install -y jq libjq-dev libonig5 libonig-dev imagemagick fonts-lato

RUN JQ_LIB_DIR=/usr/lib/x86_64-linux-gnu/libjq.so.1 cargo build --release

CMD LANG=en_US.UTF-8 LANGUAGE=en.UTF-8 cargo run --release
