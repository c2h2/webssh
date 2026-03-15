#!/bin/bash
set -e
if [ "$1" = "clean" ]; then
    cargo clean
fi
cargo build --release
echo "Built: target/release/webssh"
