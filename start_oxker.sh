#!/bin/sh
set -e

# No idea why this is solving my issue, or even where the issue is originally coming from
sleep .1

exec /usr/local/bin/oxker "$@"