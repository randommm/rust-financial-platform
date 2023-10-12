FROM rust

WORKDIR /opt

RUN cargo install sqlx-cli

COPY Cargo.toml Cargo.toml

COPY Cargo.lock Cargo.lock

COPY api/Cargo.toml api/Cargo.toml

COPY pipeline/Cargo.toml pipeline/Cargo.toml

RUN mkdir api/src && echo "fn main() {}" > api/src/main.rs

RUN mkdir pipeline/src && echo "fn main() {}" > pipeline/src/main.rs

RUN cargo build --release --locked

RUN rm -rf api/src

COPY api/src api/src

RUN rm -rf pipeline/src

COPY pipeline/src pipeline/src

RUN cargo build --release --locked

CMD cargo run --release --locked --bin rust-trading-platform-pipeline

COPY migrations migrations
