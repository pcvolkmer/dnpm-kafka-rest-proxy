FROM rust:alpine AS back-stage

RUN apk update
RUN apk add cmake make musl-dev g++ perl

WORKDIR /build
COPY Cargo.toml ./
COPY src ./src
COPY test-files ./test-files
# Run tests before release build
RUN cargo test --release
RUN cargo build --release

# Build image from scratch
FROM scratch
LABEL org.opencontainers.image.source="https://github.com/pcvolkmer/dnpm-kafka-rest-proxy"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
LABEL org.opencontainers.image.description="DNPM MTB REST Proxy für Kafka"

COPY --from=back-stage /build/target/release/dnpm-kafka-rest-proxy .
USER 65532
EXPOSE 3000
CMD ["./dnpm-kafka-rest-proxy"]
