FROM rust:1.49 AS builder

RUN mkdir -p /opt/rustyblobjectstore/build
COPY . /opt/rustyblobjectstore/build
WORKDIR /opt/rustyblobjectstore/build

RUN cargo build --release

FROM debian:stable

RUN mkdir -p /opt/rustyblobjectstore/bin
COPY --from=builder /opt/rustyblobjectstore/build/target/release/rustyblobjectstore /opt/rustyblobjectstore/bin/
