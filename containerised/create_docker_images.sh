#!/bin/bash

docker login

docker buildx prune

docker buildx build --platform linux/arm/v6,linux/arm64,linux/amd64 -t mrjackwills/oxker --push -f containerised/Dockerfile .

# Github as well?
# docker buildx build --platform linux/arm/v6,linux/arm64,linux/amd64 -t ghcr.io/mrjackwills/oxker --push -f containerised/Dockerfile .