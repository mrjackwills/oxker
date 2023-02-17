#############
## Builder ##
#############

FROM --platform=linux/amd64 rust:slim as BUILDER

ARG TARGETARCH

# These are build platform depandant, but will be ignored if not needed
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
COPY Cargo.* /usr/src/oxker/

# Set the working directory
WORKDIR /usr/src/oxker

# Install target platform (Cross-Compilation)
RUN rustup target add $(cat /.platform)

# This is a dummy build to get the dependencies cached - probably not needed - as run via a github action
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

FROM scratch AS RUNTIME

# Set an ENV that we're running in a container, so that the application can sleep for 250ms at start
ENV OXKER_RUNTIME=container

# Copy application binary from builder image
COPY --from=BUILDER /oxker /app/

# Run the application
# this is used in the application itself, to stop itself show when running from a docker container, so DO NOT EDIT
ENTRYPOINT [ "/app/oxker"]
