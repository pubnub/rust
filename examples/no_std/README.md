# `no_std`

This directory contains a project that has the examples of usage of PubNub features in `no_std` environments.

Every example is a copy of the [main examples](../) with the `no_std` boilerplate needed by the
`thumbv7m-none-eabi` target.

You can compile all the examples using the `cargo` command:

```sh
cargo build --target thumbv7m-none-eabi
```

or from the root directory

```sh
cargo build --manifest-path examples/no_std/Cargo.toml --target thumbv7m-none-eabi
```

Also, you can try to compile it for other targets!

See the [Limitations](../../README.md#limitations) section in the main readme for more information.
