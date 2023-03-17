use cucumber::{given, then, when, World};

#[derive(Debug, Default)]
struct Keyset {
    pub subkey: String,
    pub pubkey: String,
}

#[derive(Debug, Default, World)]
pub struct PubnubWorld {
    keyset: Keyset,
    last_result: String,
}

#[given("the demo keyset")]
fn set_keyset(world: &mut PubnubWorld) {
    world.keyset.pubkey = "demo".to_string();
    world.keyset.subkey = "demo".to_string();
}

#[given("a message")]
fn message_defined(_world: &mut PubnubWorld) {}

#[when("I publish a message")]
fn pubnub_publish(world: &mut PubnubWorld) {
    world.last_result = String::from("1234567890");
}

#[then(expr = "I get a timetoken {word}")]
fn check_timetoken(world: &mut PubnubWorld, timetoken: String) {
    assert_eq!(world.last_result, timetoken);
}

async fn init_server(script: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://localhost:8090/init?__contract__script__={}", script);
    let client = reqwest::Client::new();
    let body = client.get(url).send().await?.text().await?;
    Ok(body)
}

#[tokio::main]
async fn main() {
    let filtered_tags = vec!["na=rust".to_string(), "beta".to_string()];
    PubnubWorld::cucumber()
        .before(|_feature, _rule, scenario, _world| {
            futures::FutureExt::boxed(async move {
                if scenario.tags.iter().any(|t| t.starts_with("contract=")) {
                    let tag = scenario
                        .tags
                        .iter()
                        .find(|&t| t.starts_with("contract="))
                        .unwrap();
                    let splitted_values: Vec<&str> = tag.split('=').collect();
                    if !splitted_values[1].is_empty() {
                        let script_name = splitted_values[1];
                        init_server(script_name.to_string()).await.unwrap();
                    }
                }
            })
        })
        .filter_run_and_exit("tests/features", move |_, _, sc| {
            sc.tags.iter().any(|t| !filtered_tags.contains(t))
        })
        .await;
}
