#############
## Builder ##
#############

FROM --platform=linux/amd64 rust:slim as builder

ARG TARGETARCH

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="aarch64-linux-gnu-gcc"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-lgcc"
ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_MUSLEABIHF_LINKER="arm-linux-gnueabihf-ld"

COPY ./containerised/platform.sh .

RUN chmod +x ./platform.sh && ./platform.sh

RUN apt-get update && apt-get install $(cat /.compiler) -y

WORKDIR /usr/src

# Create blank project
RUN cargo new oxker

# We want dependencies cached, so copy those first
COPY Cargo.toml Cargo.lock /usr/src/oxker/

# Set the working directory
WORKDIR /usr/src/oxker

# Install target platform (Cross-Compilation)
RUN rustup target add $(cat /.platform)

# This is a dummy build to get the dependencies cached
RUN cargo build --target $(cat /.platform) --release

# Now copy in the rest of the sources
COPY src /usr/src/oxker/src/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/oxker/src/main.rs

# This is the actual application build
RUN cargo build --release --target $(cat /.platform)

RUN cp /usr/src/oxker/target/$(cat /.platform)/release/oxker /

#############
## Runtime ##
#############

FROM alpine:latest AS runtime

# Copy application binary from builder image
COPY --from=builder /oxker /usr/local/bin
COPY ./containerised/start_oxker.sh ./

# Run the application
ENTRYPOINT [ "./start_oxker.sh"]