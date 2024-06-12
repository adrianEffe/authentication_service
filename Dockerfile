# 1. Use rust official image
FROM rust:1.78.0

# 2. Copy current directory
COPY ./ ./

# 3. Build the app
RUN cargo build --release

# 4. Run the binary
CMD ["./target/release/authentication_service"]
