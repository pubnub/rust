## Examples of how to use PubNub SDK

This directory contains examples of usage the `PubNub` crate.

All simple examples can be executed with:

```sh
cargo run --example <name>
```

Each example shows concrete feature or behavior that PubNub allows you to do. [Publish](publish.rs) example that shows the easiest possible use case when you want to publish some message to the channel. Read the [main README](../README.md#Getting-started) to ensure that you satisfy all requirements to run these examples!

Some of examples require to enable certain features to work with. To use them simply add `features` switch to cargo command:

```sh
cargo run --example <name> --features="<feature1> <feature2> ..."
```

## Additional examples

You can find more additional examples of usage of PubNub SDK in sub-directories. 
Each one is separated Cargo project that allows you to find needed dependencies, used features and implementation to achieve desired usage.

## Have fun! 

