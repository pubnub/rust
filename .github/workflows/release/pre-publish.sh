#!/bin/sh

cargo install cargo-readme

# main README.md
cargo readme --no-title --no-indent-headings --no-license > README.md

