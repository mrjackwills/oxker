# FROM debian:bullseye-slim
# FROM 
FROM alpine:latest
# FROM scratch

#  DOCKER_GUID=1000 \
	# DOCKER_UID=1000 \
	# DOCKER_TIME_CONT=America \
	# DOCKER_TIME_CITY=New_York \
# ARG DOCKER_APP_USER=oxker \
	# DOCKER_APP_GROUP=docker

# ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

# RUN apt-get update \
	# && apt-get install -y ca-certificates wget \
	# && update-ca-certificates \
# RUN groupadd ${DOCKER_APP_GROUP} 
# RUN useradd --no-create-home --no-log-init ${DOCKER_APP_USER}
	# && mkdir /healthcheck /logs \
	# && chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /logs

WORKDIR /app

# COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} docker/healthcheck/health_api.sh /healthcheck

# Copy from local release destination
# COPY --chown=${DOCKER_APP_USER} target/release/oxker /app/
# COPY target/release/oxker .
# RUN mkdir app
COPY /target/x86_64-unknown-linux-musl/release/oxker ./
COPY ./start_oxker.sh ./
RUN chmod +x /app/start_oxker.sh

# Use an unprivileged user
# USER ${DOCKER_APP_USER}
ENV RUST_BACKTRACE=full
# ENTRYPOINT ["./oxker" ]
# CMD [ "./oxker"]
ENTRYPOINT ["/app/start_oxker.sh"]

# docker run --rm -ti \
#   --name=ctop \
#   --volume /var/run/docker.sock:/var/run/docker.sock:ro \
# #   quay.io/vektorlab/ctop:latest


# docker run --rm -it --volume /var/run/docker.sock:/var/run/docker.sock:ro oxker

# docker run --rm -it --volume /var/run/docker.sock:/var/run/docker.sock:ro ghcr.io/mrjackwills/oxker:latest
# could get arch, and then download appropoatley from github?

# FROM rust:latest as cargo-build

# WORKDIR /build
# ENV RUSTFLAGS="-C target-feature=+crt-static"

# COPY Cargo* ./build
# COPY src/ ./build

# RUN cargo build --release --target x86_64-unknown-linux-gnu

# #####################################


# #####################################

# FROM scratch

# COPY --from=cargo-build /build/target/x86_64-unknown-linux-gnu/release/oxker /oxker

# ENTRYPOINT [ "/oxker" ]

# FROM rust:latest AS build
# WORKDIR /oxker_build

# # Download the target for static linking.
# RUN rustup target add x86_64-unknown-linux-musl

# # Create a dummy project and build the app's dependencies.
# # If the Cargo.toml or Cargo.lock files have not changed,
# # we can use the docker build cache and skip these (typically slow) steps.
# RUN USER=root cargo new oxker --bin
# WORKDIR /oxker_build
# COPY Cargo.toml Cargo.lock ./
# # CMD ["sleep", "6000"]

# # RUN cargo build --release

# # Copy the source and build the application.
# COPY src ./src/
# RUN cargo install --target x86_64-unknown-linux-musl --path .

# # Copy the statically-linked binary into a scratch container.
# FROM scratch
# COPY --from=build /oxker_build/bin/oxker .
# # USER 1000
# CMD ["./oxker"]

# cross build --target x86_64-unknown-linux-musl --release

# rustup target add x86_64-unknown-linux-musl
# cargo build --release --target=x86_64-unknown-linux-musl