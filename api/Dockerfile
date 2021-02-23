FROM rust:1.50 AS build

WORKDIR /usr/src/api
COPY Cargo.* .
COPY src src

RUN rustup component add rustfmt
RUN cargo install --path . -j 4

FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/local/cargo/bin/api /usr/local/bin/api
CMD ["api"]