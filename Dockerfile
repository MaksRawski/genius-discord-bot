FROM rust:1.90

RUN apt-get update && apt-get install -y -f openssl ca-certificates clang fonts-lato
WORKDIR /genius-build
COPY . .

RUN cargo build --release

WORKDIR /genius
RUN mv /genius-build/target/release/genius /genius/
RUN rm -r /genius-build

CMD /genius/genius
