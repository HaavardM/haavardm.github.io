FROM rust:1.51-alpine AS build

WORKDIR /usr/src/api
RUN apk add --no-cache musl-dev protoc
COPY Cargo.* .
RUN rustup component add rustfmt
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release -j 4


COPY src src
RUN cargo build --release -j 4

FROM alpine
RUN apk add --update --no-cache ca-certificates
COPY --from=build /usr/src/api/target/release/api /usr/local/bin/api
CMD ["api"]