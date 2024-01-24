use cucumber::gherkin::Table;
use cucumber::{codegen::Regex, gherkin::Step, then, when};
use futures::{select_biased, FutureExt, StreamExt};
use log::Log;
use std::collections::HashMap;
use std::fs::read_to_string;

use crate::clear_log_file;
use crate::common::PubNubWorld;
use pubnub::core::RequestRetryConfiguration;
use pubnub::subscribe::{
    EventEmitter, EventSubscriber, Presence, Subscriber, SubscriptionOptions, SubscriptionSet,
};

/// Extract list of events and invocations from log.
fn events_and_invocations_history() -> Vec<Vec<String>> {
    let mut lines: Vec<Vec<String>> = Vec::new();
    let written_log =
        read_to_string("tests/logs/log.txt").expect("Unable to read history from log");
    let event_regex = Regex::new(r" DEBUG .* Processing event: (.+)$").unwrap();
    let invocation_regex = Regex::new(r" DEBUG .* Received invocation: (.+)$").unwrap();
    let known_events = [
        "JOINED",
        "LEFT",
        "LEFT_ALL",
        "HEARTBEAT_SUCCESS",
        "HEARTBEAT_FAILURE",
        "HEARTBEAT_GIVEUP",
        "RECONNECT",
        "DISCONNECT",
        "TIMES_UP",
    ];
    let known_invocations = [
        "HEARTBEAT",
        "DELAYED_HEARTBEAT",
        "CANCEL_DELAYED_HEARTBEAT",
        "LEAVE",
        "WAIT",
        "CANCEL_WAIT",
    ];

    for line in written_log.lines() {
        if !line.contains(" DEBUG ") {
            continue;
        }

        if let Some(matched) = event_regex.captures(line) {
            let (_, [captured]) = matched.extract();
            if known_events.contains(&captured) {
                lines.push(["event".into(), captured.into()].to_vec());
            }
        }

        if let Some(matched) = invocation_regex.captures(line) {
            let (_, [captured]) = matched.extract();
            if known_invocations.contains(&captured) {
                lines.push(["invocation".into(), captured.into()].to_vec());
            }
        }
    }

    lines
}

fn event_occurrence_count(history: Vec<Vec<String>>, event: String) -> usize {
    history
        .iter()
        .filter(|pair| pair[0].eq("event") && pair[1].eq(&event))
        .count()
}

fn invocation_occurrence_count(history: Vec<Vec<String>>, invocation: String) -> usize {
    history
        .iter()
        .filter(|pair| pair[0].eq("invocation") && pair[1].eq(&invocation))
        .count()
}

/// Match list of events and invocations pairs to table defined in step.
fn match_history_to_feature(history: Vec<Vec<String>>, table: &Table) {
    log::logger().flush();

    (!table.rows.iter().skip(1).eq(history.iter())).then(|| {
        let expected = {
            table
                .rows
                .iter()
                .skip(1)
                .map(|pair| format!("    ({}) {}", pair[0], pair[1]))
                .collect::<Vec<String>>()
                .join("\n")
        };
        let received = {
            history
                .iter()
                .skip(1)
                .map(|pair| format!("    ({}) {}", pair[0], pair[1]))
                .collect::<Vec<String>>()
                .join("\n")
        };

        panic!(
            "Unexpected set of events and invocations:\n  -expected:\n{}\n\n  -got:\n{}\n",
            expected, received
        )
    });
}

#[when(regex = r"^I join '(.*)', '(.*)', '(.*)' channels( with presence)?$")]
async fn join(
    world: &mut PubNubWorld,
    channel_a: String,
    channel_b: String,
    channel_c: String,
    with_presence: String,
) {
    // Start recording subscription session.
    clear_log_file();

    world.pubnub = Some(world.get_pubnub(world.keyset.to_owned()));
    let Some(client) = world.pubnub.clone() else {
        panic!("Unable to get PubNub client instance");
    };

    let options =
        (!with_presence.is_empty()).then_some(vec![SubscriptionOptions::ReceivePresenceEvents]);
    let subscriptions =
        vec![channel_a, channel_b, channel_c]
            .iter()
            .fold(HashMap::new(), |mut acc, channel| {
                acc.insert(
                    channel.clone(),
                    client.channel(channel).subscription(options.clone()),
                );
                acc
            });
    let subscription = SubscriptionSet::new_with_subscriptions(
        subscriptions.values().cloned().collect(),
        options.clone(),
    );
    subscription.subscribe(None);
    world.subscription = Some(subscription);
    world.subscriptions = Some(subscriptions);
}

#[then(regex = r"^I wait '([0-9]+)' seconds$")]
async fn wait(_world: &mut PubNubWorld, delay: u64) {
    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
}

#[then(regex = r"^I leave '(.*)' and '(.*)' channels( with presence)?$")]
async fn leave(
    world: &mut PubNubWorld,
    channel_a: String,
    channel_b: String,
    with_presence: String,
) {
    let mut subscription = world.subscription.clone().unwrap();
    let subscriptions = world.subscriptions.clone().unwrap().iter().fold(
        vec![],
        |mut acc, (channel, subscription)| {
            if channel.eq(&channel_a) || channel.eq(&channel_b) {
                acc.push(subscription.clone())
            }
            acc
        },
    );

    subscription -= SubscriptionSet::new_with_subscriptions(subscriptions, None);
}

#[then("I wait for getting Presence joined events")]
async fn wait_presence_join(world: &mut PubNubWorld) {
    let mut subscription = world.subscription.clone().unwrap().presence_stream();

    select_biased! {
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)).fuse() => panic!("No service response"),
        update = subscription.next().fuse() => {
            match update.clone().unwrap() {
                Presence::Join { .. } => {},
                _ => panic!("Unexpected presence update received: {update:?}"),
            };

            subscription.next().await;
            subscription.next().await;
        }
    }
}

#[then("I receive an error in my heartbeat response")]
async fn receive_an_error_heartbeat_retry(world: &mut PubNubWorld) {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let history = events_and_invocations_history();
    let expected_retry_count: usize = usize::from(match &world.retry_policy.clone().unwrap() {
        RequestRetryConfiguration::Linear { max_retry, .. }
        | RequestRetryConfiguration::Exponential { max_retry, .. } => *max_retry,
        _ => 0,
    });

    assert_eq!(
        event_occurrence_count(history.clone(), "HEARTBEAT_FAILURE".into()),
        expected_retry_count + 1
    );
    assert_eq!(
        event_occurrence_count(history, "HEARTBEAT_GIVEUP".into()),
        1
    );
}

#[then("I don't observe any Events and Invocations of the Presence EE")]
async fn event_engine_history_empty(_world: &mut PubNubWorld, step: &Step) {
    assert_eq!(events_and_invocations_history().len(), 0);
}

#[then("I observe the following Events and Invocations of the Presence EE:")]
async fn event_engine_history(_world: &mut PubNubWorld, step: &Step) {
    let history = events_and_invocations_history();

    if let Some(table) = step.table.as_ref() {
        match_history_to_feature(history, table);
    } else {
        panic!("Unable table content.")
    }
}
