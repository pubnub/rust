use crate::common::PubNubWorld;
use crate::{clear_log_file, scenario_name};
use cucumber::gherkin::Table;
use cucumber::{codegen::Regex, gherkin::Step, then, when};
use futures::{select_biased, FutureExt, StreamExt};
use pubnub::core::RequestRetryConfiguration;
use pubnub::subscribe::{EventEmitter, EventSubscriber};
use std::fs::read_to_string;

/// Extract list of events and invocations from log.
fn events_and_invocations_history() -> Vec<Vec<String>> {
    let mut lines: Vec<Vec<String>> = Vec::new();
    let written_log =
        read_to_string("tests/logs/log.txt").expect("Unable to read history from log");
    let event_regex = Regex::new(r" DEBUG .* Processing event: (.+)$").unwrap();
    let invocation_regex = Regex::new(r" DEBUG .* Received invocation: (.+)$").unwrap();
    let known_events = [
        "SUBSCRIPTION_CHANGED",
        "SUBSCRIPTION_RESTORED",
        "HANDSHAKE_SUCCESS",
        "HANDSHAKE_FAILURE",
        "HANDSHAKE_RECONNECT_SUCCESS",
        "HANDSHAKE_RECONNECT_FAILURE",
        "HANDSHAKE_RECONNECT_GIVEUP",
        "RECEIVE_SUCCESS",
        "RECEIVE_FAILURE",
        "RECEIVE_RECONNECT_SUCCESS",
        "RECEIVE_RECONNECT_FAILURE",
        "RECEIVE_RECONNECT_GIVEUP",
        "DISCONNECT",
        "RECONNECT",
        "UNSUBSCRIBE_ALL",
    ];
    let known_invocations = [
        "HANDSHAKE",
        "CANCEL_HANDSHAKE",
        "HANDSHAKE_RECONNECT",
        "CANCEL_HANDSHAKE_RECONNECT",
        "RECEIVE_MESSAGES",
        "CANCEL_RECEIVE_MESSAGES",
        "RECEIVE_RECONNECT",
        "CANCEL_RECEIVE_RECONNECT",
        "EMIT_STATUS",
        "EMIT_MESSAGES",
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

#[when("I subscribe")]
async fn subscribe(world: &mut PubNubWorld) {
    // Start recording subscription session.
    clear_log_file();
    world.pubnub = Some(world.get_pubnub(world.keyset.to_owned()));

    world.pubnub.clone().map(|pubnub| {
        let subscription = pubnub.subscription(Some(&["test"]), None, None);
        subscription.subscribe(None);
        world.subscription = Some(subscription);
    });
}

#[when(regex = r"^I subscribe with timetoken ([0-9]+)$")]
async fn subscribe_with_timetoken(world: &mut PubNubWorld, timetoken: u64) {
    // Start recording subscription session.
    clear_log_file();
    world.pubnub = Some(world.get_pubnub(world.keyset.to_owned()));

    world.pubnub.clone().map(|pubnub| {
        let subscription = pubnub.subscription(Some(&["test"]), None, None);
        subscription.subscribe(Some(timetoken.to_string().into()));
        world.subscription = Some(subscription);
    });
}

#[then("I receive the message in my subscribe response")]
async fn receive_message(world: &mut PubNubWorld) {
    let mut subscription = world.subscription.clone().unwrap().messages_stream();

    select_biased! {
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)).fuse() => panic!("No service response"),
        _ = subscription.next().fuse() => println!("Message received from server")
    }
}

#[then("I receive an error in my subscribe response")]
async fn receive_an_error_subscribe_retry(world: &mut PubNubWorld) {
    let mut subscription = world.subscription.clone().unwrap().messages_stream();

    select_biased! {
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)).fuse() => log::debug!("Two second is done"),
        _ = subscription.next().fuse() => panic!("Message update from server")
    }

    let expected_retry_count: usize = usize::from(match &world.retry_policy.clone().unwrap() {
        RequestRetryConfiguration::Linear { max_retry, .. }
        | RequestRetryConfiguration::Exponential { max_retry, .. } => *max_retry,
        _ => 0,
    });

    let handshake_test = scenario_name(world).to_lowercase().contains("handshake");
    let history = events_and_invocations_history();
    let normal_operation_name = {
        if handshake_test {
            "HANDSHAKE_FAILURE"
        } else {
            "RECEIVE_FAILURE"
        }
    };
    let reconnect_operation_name = {
        if handshake_test {
            "HANDSHAKE_RECONNECT_FAILURE"
        } else {
            "RECEIVE_RECONNECT_FAILURE"
        }
    };
    let give_up_operation_name = {
        if handshake_test {
            "HANDSHAKE_RECONNECT_GIVEUP"
        } else {
            "RECEIVE_RECONNECT_GIVEUP"
        }
    };

    assert_eq!(
        event_occurrence_count(history.clone(), normal_operation_name.into()),
        1,
        "{normal_operation_name} should appear at least once"
    );
    assert_eq!(
        event_occurrence_count(history.clone(), reconnect_operation_name.into()),
        expected_retry_count,
        "{reconnect_operation_name} should appear {expected_retry_count} times"
    );
    assert_eq!(
        event_occurrence_count(history, give_up_operation_name.into()),
        1,
        "{give_up_operation_name} should appear at least once"
    );
}

#[then("I observe the following:")]
async fn event_engine_history(_world: &mut PubNubWorld, step: &Step) {
    let history = events_and_invocations_history();

    if let Some(table) = step.table.as_ref() {
        match_history_to_feature(history, table);
    } else {
        panic!("Unable table content.")
    }
}
