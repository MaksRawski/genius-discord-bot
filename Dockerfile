FROM rust:latest

ENV MAGICK_VERSION 7.1.0-39

RUN git clone https://github.com/ImageMagick/ImageMagick && cd ImageMagick \
 && git checkout $MAGICK_VERSION \
 && ./configure --with-magick-plus-plus=no --with-perl=no \
 && make \
 && make install \
 && cd .. \
 && rm -r ImageMagick


WORKDIR /usr/src/genius-bot
COPY . .

RUN apt-get update -y
RUN apt-get install jq fonts-lato -y


# FROM setup as build
# RUN cargo build --release

# CMD cargo run --release
