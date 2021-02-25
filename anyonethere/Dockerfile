FROM rust:1.50 AS build

WORKDIR /usr/src/api
COPY Cargo.* .
RUN rustup component add rustfmt
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release


COPY src src
RUN cargo build --release -j 4

FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/src/api/target/release/api /usr/local/bin/api
CMD ["api"]