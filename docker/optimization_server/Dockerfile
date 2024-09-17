# Use an official Rust image to build the project
FROM rust:latest as builder
WORKDIR /usr/src/app

# Install build dependencies (build-essential and cmake)
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake

# Copy the entire project and build it
COPY rust/optimization_server .
RUN cargo build --release

# Use a minimal image for running the built server
FROM rust:slim
WORKDIR /usr/src/app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/optimization_server .

# Expose the necessary port
EXPOSE 8080

# Run the TCP server on port 8080
CMD ["./optimization_server"]
