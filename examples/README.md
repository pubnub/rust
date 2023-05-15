## Examples of how to use PubNub SDK

This directory contains usage examples of the `PubNub` crate.

You can execute all simple examples can like this:

```sh
cargo run --example <name>
```

Each example shows a concrete PubNub feature or behavior. The [Publish](publish.rs) example shows the easiest possible use case of publishing a message to a channel. Read the main [README](../README.md#Getting-started) to ensure you meet all requirements to run these examples!

Some examples require specific features to be enabled. To enable those features, add the `features` switch to the `cargo` command:

```sh
cargo run --example <name> --features="<feature1> <feature2> ..."
```

## Additional examples

You can find more examples in the subdirectories. Each one is a separate Cargo project that allows you to understand the necessary dependencies, used features, and implementation details.

Have fun!
