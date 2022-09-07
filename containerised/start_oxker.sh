#!/bin/sh
set -e

# Without this sleep, the docker image will instantly close
# No idea why this is solving my issue, or even where the issue is originally coming from
sleep .1

exec /usr/local/bin/oxker "$@"