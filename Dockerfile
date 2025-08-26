FROM rust:alpine AS build-stage

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
LABEL org.opencontainers.image.source="https://github.com/pcvolkmer/mv64e-rest-to-kafka-gateway"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
LABEL org.opencontainers.image.description="Send MV64e HTTP requests with DNPM V2.1 payload to a Kafka broker"

COPY --from=build-stage /build/target/release/mv64e-rest-to-kafka-gateway .
USER 65532
EXPOSE 3000
CMD ["./mv64e-rest-to-kafka-gateway"]
