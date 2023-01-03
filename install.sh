#!/bin/bash

case "$(arch)" in
  x86_64) SUFFIX="x86_64";;
  aarch64) SUFFIX="aarch64";;
  armv6l) SUFFIX="armv6";;
esac

if [ -n "$SUFFIX" ]; then
  wget "https://github.com/mrjackwills/oxker/releases/latest/download/oxker_linux_${SUFFIX}.tar.gz"
  tar xzvf "oxker_linux_${SUFFIX}.tar.gz" oxker
  install -Dm 755 oxker -t "${HOME}/.local/bin"
  rm "oxker_linux_${SUFFIX}.tar.gz" oxker
fi
