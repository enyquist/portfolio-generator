# Stage 1: Plan stage
FROM rust:slim AS planner
WORKDIR /usr/src/app
RUN cargo install cargo-chef
COPY rust/optimization_server .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cache dependencies
FROM rust:slim AS cacher
WORKDIR /usr/src/app

# Install build dependencies (build-essential and cmake)
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake

RUN cargo install cargo-chef
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: Build stage
FROM rust:slim AS builder
WORKDIR /usr/src/app

# Install build dependencies (build-essential and cmake)
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake

COPY rust/optimization_server .
COPY --from=cacher /usr/src/app/target/release/deps target/release/deps
RUN cargo build --release
RUN strip /usr/src/app/target/release/optimization_server

# Stage 4: Runtime stage
FROM ubuntu:22.04
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/optimization_server .

# Expose the necessary port
EXPOSE 8080

# Run the TCP server on port 8080
CMD ["./optimization_server"]
