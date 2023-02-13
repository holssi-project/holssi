FROM rust:1.67-alpine3.17 as builder

RUN apk add --no-cache musl-dev 

RUN cargo new --bin app
WORKDIR /app
COPY ./cli/Cargo.toml ./Cargo.toml
RUN cargo build --release && rm src/*.rs ./target/release/deps/holssi*

COPY cli .
RUN cargo build --release

FROM alpine:3.17 as runner

RUN mkdir /in && mkdir /out

RUN apk add --no-cache wine git nodejs npm zip p7zip
RUN ln -s /usr/bin/wine64 /usr/bin/wine

COPY boilerplate /boilerplate

RUN cd /boilerplate && npm install

COPY --from=builder /app/target/release/holssi /usr/local/bin/

ENV USE_SYSTEM_7ZA=true

ENTRYPOINT [ "holssi", "--out", "/out", "--boilerplate", "/boilerplate", "--local","--no-copy", "--no-npm-install", "/in/project.ent" ]