#!/bin/bash

VERSION=$(grep '^version =' Cargo.toml | cut -d '=' -f2 | tr -d '"')
echo $VERSION

RUSTFLAGS="-C linker=arm-linux-gnueabihf-gcc --verbose" cargo build --release --target armv7-unknown-linux-gnueabihf -o ka-server-arm-$VERSION.bin