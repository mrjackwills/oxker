#############
## Builder ##
#############

FROM rust:1.63.0-slim as builder

WORKDIR /usr/src

# Create blank project
RUN USER=root cargo new oxker

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/oxker/

# Set the working directory
WORKDIR /usr/src/oxker

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release

# Now copy in the rest of the sources
COPY src /usr/src/oxker/src/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/oxker/src/main.rs

# This is the actual application build.
RUN cargo build --target x86_64-unknown-linux-musl --release

#############
## Runtime ##
#############

FROM alpine:latest AS runtime

# Copy application binary from builder image
COPY --from=builder /usr/src/oxker/target/x86_64-unknown-linux-musl/release/oxker /usr/local/bin
COPY start_oxker.sh ./
RUN chmod +x /start_oxker.sh

# Run the application
ENTRYPOINT [ "./start_oxker.sh"]

# docker run --rm -it --volume /var/run/docker.sock:/var/run/docker.sock:ro oxker