<img width="1920" height="600" alt="rust-header" src="https://github.com/user-attachments/assets/d8ef34f3-bbe3-49cb-bbbd-3accea563444" />

# PubNub Rust SDK

[![crates.io](https://img.shields.io/crates/v/pubnub.svg)](https://crates.io/crates/pubnub)

PubNub provides global infrastructure for real-time, interactive applications.

Publish and receive messages in Rust. Use this SDK for Rust server, desktop, WebAssembly, and embedded applications.

[Documentation](https://www.pubnub.com/docs/sdks/rust) · [API reference](https://www.pubnub.com/docs/sdks/rust/api-reference/publish-and-subscribe) · [Changelog](https://www.pubnub.com/docs/sdks/rust/changelog) · [Support](https://support.pubnub.com/)

## Requirements

| Requirement | Supported version or setup |
| --- | --- |
| Rust | [Rust 1.65.0 or later](https://www.pubnub.com/docs/sdks/rust/platform-support) |
| Targets | [Windows, macOS, Unix/Linux, WebAssembly, and embedded systems](https://www.pubnub.com/docs/sdks/rust/platform-support) |
| Cargo dependencies | The async Quickstart uses PubNub, Tokio, Futures, and serde_json. |

## Installation

Add the following to `Cargo.toml`:

```toml
[dependencies]
pubnub = "0.8.0"
futures = "0.3"
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

For other installation methods, see [environment setup](https://www.pubnub.com/docs/sdks/rust).

## Quickstart

This example runs in a Tokio-based Rust command-line application. It subscribes to `hello_world`, publishes one message, and prints the received text.

### Get your keys

1. Open the [PubNub Admin Portal](https://admin.pubnub.com/).
2. Create an app and a keyset for development, or select an existing development keyset.
3. Copy its publish key and subscribe key.

If Access Manager is enabled for your keyset, obtain a token from your trusted backend and call `pubnub.set_token(token)` before accessing protected resources. The issued token is passed to the client. Keep the secret key on the backend and never embed it in distributed client code.

### Send and receive a message

Create a Cargo binary project and replace `src/main.rs` with:

Replace the key placeholders. Use a User ID that identifies the user or device in your app.

```rust
use futures::StreamExt;
use pubnub::{
    subscribe::{ConnectionStatus, EventEmitter, EventSubscriber, Subscriber},
    Keyset, PubNubClientBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pubnub = PubNubClientBuilder::with_reqwest_transport()
        .with_keyset(Keyset {
            subscribe_key: "YOUR_SUBSCRIBE_KEY",
            publish_key: Some("YOUR_PUBLISH_KEY"),
            secret_key: None,
        })
        .with_user_id("hello-world-user")
        .build()?;

    let subscription = pubnub.channel("hello_world").subscription(None);

    // Create the streams before subscribing so no event is missed.
    let mut statuses = pubnub.status_stream();
    let mut messages = subscription.messages_stream();

    subscription.subscribe();

    // Wait until the subscription is connected before publishing.
    while let Some(status) = statuses.next().await {
        match status {
            ConnectionStatus::Connected => break,
            ConnectionStatus::ConnectionError(error) => return Err(error.into()),
            _ => {}
        }
    }

    pubnub
        .publish_message("Hello world")
        .channel("hello_world")
        .execute()
        .await?;

    if let Some(message) = messages.next().await {
        let text: String = serde_json::from_slice(&message.data)?;
        println!("{text}");
    }

    subscription.unsubscribe();
    pubnub.terminate();

    Ok(())
}
```

### Run the example

```
cargo run
```

The terminal should show:

```
Hello world
```

The example calls `subscription.unsubscribe()` and `pubnub.terminate()` after it receives the message. In a long-running application, call them when you no longer need the subscription or when the client shuts down.

For a complete application, see the [getting started guide](https://www.pubnub.com/docs/sdks/rust).

## Next steps

| Task | Documentation |
| --- | --- |
| Configure the client | [Configuration](https://www.pubnub.com/docs/sdks/rust/api-reference/configuration) |
| Work with subscriptions and messages | [Publish & Subscribe](https://www.pubnub.com/docs/sdks/rust/api-reference/publish-and-subscribe) |
| Check channel occupancy | [Presence](https://www.pubnub.com/docs/sdks/rust/api-reference/presence) |
| Secure resources | [Access Manager](https://www.pubnub.com/docs/sdks/rust/api-reference/access-manager) |

## Build with an AI coding assistant

The [PubNub MCP server](https://www.pubnub.com/docs/ai/pubnub-mcp-server) gives an AI coding assistant access to PubNub SDK documentation and PubNub APIs. Connect the assistant to the hosted server at `https://mcp.pubnub.com`, or run `npx @pubnub/mcp@latest` locally.

The [server repository](https://github.com/pubnub/pubnub-mcp-server) has setup steps for VS Code, Cursor, Claude Code, Claude Desktop, Codex, Gemini CLI, and OpenCode.

## Before production

Use Access Manager to grant each client the permissions it needs. Keep the secret key on your backend. Never include it in a distributed client.

Each PubNub client holds its own transport and subscription state. Create one client per application or service and reuse it. Call `subscription.unsubscribe()` to stop an individual subscription, and call `pubnub.terminate()` when the client is shutting down.

Live delivery through PubNub SDKs is at-most-once. A subscriber can miss messages while disconnected or if its buffer overflows. The current Rust SDK documentation does not expose a Message Persistence client API; check [Rust Feature Support](https://www.pubnub.com/docs/sdks/rust/feature-support) before designing history-based recovery around this SDK.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| The Quickstart has unresolved imports for Tokio, Futures, or serde_json | Add every dependency shown in the Quickstart to `Cargo.toml`; the runnable example needs more than the `pubnub` crate alone. See the [Rust SDK guide](https://www.pubnub.com/docs/sdks/rust). |
| A publish succeeds but no message appears | Check the message handler, subscription readiness, keyset, and channel name. |
| A PubNub API is unavailable at compile time | Enable the crate feature that provides that API. See [Rust Configuration](https://www.pubnub.com/docs/sdks/rust/api-reference/configuration). |
| Extra clients or duplicate messages during development | Stop the previous process before restarting. Each live client that subscribes to a channel receives its own copy of every message, so a leftover process doubles them. |

For setup help, see [troubleshooting](https://www.pubnub.com/docs/sdks/rust). Check [network status](https://status.pubnub.com/) for service incidents.

## Releases

Read the [changelog](https://www.pubnub.com/docs/sdks/rust/changelog) before upgrading.

For a major version change, review the [Rust SDK changelog](https://www.pubnub.com/docs/sdks/rust/changelog) for breaking changes before upgrading.

## Support and contributions

For setup or account help, contact [PubNub Support](https://support.pubnub.com/).

For a reproducible SDK bug, use [GitHub Issues](https://github.com/pubnub/rust/issues). Include the SDK version, runtime, and a small reproduction with credentials removed.

Before opening a pull request, run `cargo fmt --check`, `cargo clippy --features full -- -D warnings`, and `cargo test --features full`. Include tests for behavioral changes.

## License

See the [PubNub Software Development Kit License Agreement](https://github.com/pubnub/rust/blob/master/LICENSE).
