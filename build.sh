#!/bin/bash

set -e

APP=queue
VERSION=$(cargo metadata --no-deps --format-version=1 | jq -r .packages[0].version)

export CC_x86_64_unknown_linux_musl=x86_64-unknown-linux-musl-gcc
export CXX_x86_64_unknown_linux_musl=x86_64-unknown-linux-musl-g++
export AR_x86_64_unknown_linux_musl=x86_64-unknown-linux-musl-ar
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-unknown-linux-musl-gcc

cargo build --release --target x86_64-unknown-linux-musl

aws ecr get-login-password --region eu-west-1 | docker login --username AWS --password-stdin 938562635226.dkr.ecr.eu-west-1.amazonaws.com

docker build -t cubr-services/$APP .
docker tag cubr-services/$APP 938562635226.dkr.ecr.eu-west-1.amazonaws.com/cubr-services:$APP-$VERSION
docker push 938562635226.dkr.ecr.eu-west-1.amazonaws.com/cubr-services:$APP-$VERSION