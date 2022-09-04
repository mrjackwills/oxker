#!/bin/sh
set -e

# No idea why this is sloving my issue, or even where the issue is originally coming from
sleep 1

exec ./oxker "$@"