# Stage 1: Build the Rust project
FROM rust:1.70 as builder

WORKDIR /app

COPY . .

# Build the application in release mode
RUN cargo build --release

# Stage 2: Create the runtime container
FROM debian:bullseye-slim

# Copy only the built binary from the builder stage
COPY --from=builder /app/target/release/dvizh_bot /usr/local/bin/dvizh_bot

# Specify the command to run the application
CMD ["dvizh_bot"]
