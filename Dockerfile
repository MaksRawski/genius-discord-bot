FROM rust

RUN apt-get update
RUN apt-get install -y -f openssl ca-certificates clang jq libjq-dev libonig5 libonig-dev fonts-lato

RUN wget https://imagemagick.org/archive/binaries/magick -O magick
RUN chmod +x ./magick; ./magick --appimage-extract
RUN mv /squashfs-root/usr/lib/pkgconfig/*       /usr/lib/x86_64-linux-gnu/pkgconfig/
RUN mv /squashfs-root/usr/lib/*                 /usr/lib/
RUN mv /squashfs-root/usr/include/ImageMagick-7 /usr/include/

WORKDIR /genius
COPY . .

ENV JQ_LIB_DIR=/usr/lib/x86_64-linux-gnu/libjq.so
RUN cargo build --release

CMD cargo run --release
