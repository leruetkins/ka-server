#!/bin/bash

RUSTFLAGS="-C linker=arm-linux-gnueabihf-gcc --verbose" cargo build --release --target armv7-unknown-linux-gnueabihf