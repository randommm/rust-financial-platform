FROM ubuntu

RUN apt-get update

RUN apt-get install -y \
    curl \
    clang \
    gcc \
    g++ \
    zlib1g-dev \
    libmpc-dev \
    libmpfr-dev \
    libgmp-dev \
    git \
    cmake \
    pkg-config \
    libssl-dev \
    build-essential

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s - -y

ENV PATH=/root/.cargo/bin:${PATH}

WORKDIR /opt

RUN cargo install sqlx-cli

COPY Cargo.toml Cargo.toml

COPY Cargo.lock Cargo.lock

COPY api/Cargo.toml api/Cargo.toml

COPY pipeline/Cargo.toml pipeline/Cargo.toml

RUN mkdir api/src && echo "fn main() {}" > api/src/main.rs

RUN mkdir pipeline/src && echo "fn main() {}" > pipeline/src/main.rs

RUN cargo build --release --locked --bin rust-trading-platform-api

RUN cargo build --release --locked --bin rust-trading-platform-pipeline

RUN rm -rf api/src

COPY api/src api/src

RUN rm -rf pipeline/src

COPY pipeline/src pipeline/src

CMD cargo run --release --locked --bin rust-trading-platform-pipeline

COPY migrations migrations
