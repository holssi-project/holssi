FROM rust:1.67-buster as builder

RUN cargo new --bin app
WORKDIR /app
COPY ./cli/Cargo.toml ./Cargo.toml
RUN cargo build --release && rm src/*.rs ./target/release/deps/holssi*

COPY cli .
RUN cargo build --release

FROM electronuserland/builder:wine as runner

RUN mkdir /in && mkdir /out

COPY boilerplate /boilerplate

RUN cd /boilerplate && npm install
#npm run dist -- --win --mac --x64 --arm64 && rm -rf /boilerplate/dist

COPY --from=builder /app/target/release/holssi /usr/local/bin/

# ENV USE_SYSTEM_7ZA=true

ENTRYPOINT [ "holssi", "--out", "/out", "--boilerplate", "/boilerplate", "--local","--no-copy", "--no-npm-install", "/in/project.ent" ]