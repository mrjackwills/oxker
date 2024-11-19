#!/bin/sh
# Used in Docker build to set platform dependent variables

case $TARGETARCH in

"amd64")
	echo "x86_64-unknown-linux-musl" >/.target
	echo "" >/.compiler
	;;
"arm64")
	echo "aarch64-unknown-linux-musl" >/.target
	echo "gcc-aarch64-linux-gnu" >/.compiler
	;;
"arm")
	echo "arm-unknown-linux-musleabihf" >/.target
	echo "gcc-arm-linux-gnueabihf" >/.compiler
	;;
esac
