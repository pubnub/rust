#!/bin/sh

# TODO: check if it is even possible with our bot
# Now it will be called right after change log and version bump will be done.

cargo install cargo-readme

# main README.md
cargo readme --no-title --no-indent-headings --no-license > README.md

