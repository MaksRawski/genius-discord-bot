FROM rustlang/rust:nightly as builder

WORKDIR /usr/src/genius-bot
COPY . .
RUN apt-get update -y
RUN apt-get install jq imagemagick fonts-lato -y

RUN cargo build --release

CMD cargo run --release
