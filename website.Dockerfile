FROM rust:1.74-buster as builder

RUN cargo new --bin app
WORKDIR /app
COPY ./cli/Cargo.toml ./Cargo.toml
RUN cargo build --release --features=website && rm src/*.rs ./target/release/deps/holssi*

COPY cli .
RUN cargo build --release --features=website

FROM electronuserland/builder:wine as runner

COPY boilerplate /boilerplate

RUN cd /boilerplate && npm install
RUN cd /boilerplate && npm run dist -- --win --mac --x64 --arm64 && rm -rf /boilerplate/dist

RUN apt-get update && apt-get install -y zip && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/holssi /usr/local/bin/

ENTRYPOINT [ "holssi", "--boilerplate", "/boilerplate", "--local", "--no-copy", "--no-npm-install" ]
