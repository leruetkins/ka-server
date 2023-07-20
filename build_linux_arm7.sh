#!/bin/bash

VERSION=$(grep '^version =' Cargo.toml | cut -d '=' -f2 | tr -d '"' | tr -d '[:space:]')

echo $VERSION

RUSTFLAGS="-C linker=arm-linux-gnueabihf-gcc --verbose" cargo build --release --target armv7-unknown-linux-gnueabihf

mv ./target/armv7-unknown-linux-gnueabihf/release/ka-server ./target/armv7-unknown-linux-gnueabihf/release/ka-server-arm-$VERSION.bin
