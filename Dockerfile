FROM rust

WORKDIR /usr/src/genius-bot
COPY . .
RUN apt-get update -y
RUN apt-get install -y jq imagemagick fonts-lato

RUN cargo build --release

CMD cargo run --release
