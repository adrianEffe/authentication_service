FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /authentication_service

FROM chef AS planner
# Copy source code from previous stage
COPY . .
# Generate info for caching dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /authentication_service/recipe.json recipe.json
# Build & cache dependencies
RUN cargo chef cook --release --recipe-path recipe.json
# Copy source code from previous stage
COPY . .
# Build application
RUN cargo build --release 

# Create a new stage with a minimal image
FROM debian:bookworm-slim AS runtime
WORKDIR /authentication_service
COPY --from=builder /authentication_service/target/release/authentication_service /usr/local/bin
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/authentication_service"]
