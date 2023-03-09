#!/bin/sh

# TODO: check if it is even possible with our bot
# Now it will be called right after change log and version bump will be done.

# main README.md
cargo readme -r crates/pubnub --no-title --no-indent-headings --no-license > README.md

# all crates README.md
for crate in crates/*; do
    if [ $crate = "crates/pubnub" ]; then
        continue
    fi

    cargo readme -r $crate --no-title --no-indent-headings --no-license > $crate/README.md
done

