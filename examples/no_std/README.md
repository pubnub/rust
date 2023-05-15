# `no_std`

This directory contains project that contains examples with usage of PubNub features with `no_std` environments.

Every example here is a rewritten copy of the [main examples](../) with the `no_std` boilerplate needed by 
`thumbv7m-none-eabi` target.

You can compile all the examples using cargo command:

```sh
cargo build --target thumbv7m-none-eabi
```

or from the root directory

```sh
cargo build --manifest-path examples/no_std/Cargo.toml --target thumbv7m-none-eabi
```

Also you can try to compile it for other targets!
See the [limitations paragraph](../../README.md#limitations) in main readme.

