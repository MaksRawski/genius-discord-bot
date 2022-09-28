FROM rust

WORKDIR /genius
COPY . .

RUN apt-get update

# snippet below fixes segfault when libc-bin is processed
# after any apt-get install when building through CI
# https://github.com/microsoft/WSL/issues/4760#issuecomment-642715044
RUN mv /var/lib/dpkg/info/libc-bin.* /tmp/
RUN dpkg --remove --force-remove-reinstreq --force-remove-essential --force-depends libc-bin
RUN dpkg --purge libc-bin
RUN apt-get install -y libc-bin
RUN mv /tmp/libc-bin.* /var/lib/dpkg/info/

RUN apt-get install -y -f jq libjq-dev libonig5 libonig-dev openssl ca-certificates fonts-lato imagemagick

RUN JQ_LIB_DIR=/usr/lib/x86_64-linux-gnu/libjq.so.1 cargo build --release

CMD LANG=en_US.UTF-8 LANGUAGE=en.UTF-8 cargo run --release
