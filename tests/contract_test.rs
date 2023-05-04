use cucumber::{writer, World, WriterExt};
use std::fs::File;

mod access;
mod common;
mod publish;
use common::PubNubWorld;

async fn init_server(script: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://localhost:8090/init?__contract__script__={}", script);
    let client = reqwest::Client::new();
    let body = client.get(url).send().await?.text().await?;
    Ok(body)
}

fn get_feature_set(tags: &[String]) -> String {
    tags.iter()
        .filter(|tag| tag.starts_with("featureSet"))
        .map(|tag| tag.split('=').last().unwrap().to_string())
        .collect::<String>()
}

fn feature_allows_beta(feature: &str) -> bool {
    let features: Vec<&str> = vec!["access", "publish"];
    features.contains(&feature)
}

fn feature_allows_skipped(feature: &str) -> bool {
    let features: Vec<&str> = vec![];
    features.contains(&feature)
}

fn feature_allows_contract_less(feature: &str) -> bool {
    let features: Vec<&str> = vec!["access"];
    features.contains(&feature)
}

fn is_ignored_feature_set_tag(feature: &str, tags: &[String]) -> bool {
    let supported_features = ["access", "publish"];
    let mut ignored_tags = vec!["na=rust"];

    if !feature_allows_beta(feature) {
        ignored_tags.push("beta");
    }

    if !feature_allows_skipped(feature) {
        ignored_tags.push("skip");
    }

    ignored_tags
        .iter()
        .any(|tag| tags.contains(&tag.to_string()))
        || !supported_features.contains(&feature)
}

fn is_ignored_scenario_tag(feature: &str, tags: &[String]) -> bool {
    // If specific contract should be tested, it's name should be added below.
    let tested_contract = "";

    tags.contains(&"na=rust".to_string())
        || !feature_allows_beta(feature) && tags.iter().any(|tag| tag.starts_with("beta"))
        || !feature_allows_skipped(feature) && tags.iter().any(|tag| tag.starts_with("skip"))
        || (!feature_allows_contract_less(feature) || !tested_contract.is_empty())
            && !tags
                .iter()
                .any(|tag| tag.starts_with(format!("contract={tested_contract}").as_str()))
}

#[tokio::main]
async fn main() {
    env_logger::builder().try_init().unwrap();
    let _ = std::fs::create_dir_all("tests/reports");
    let file: File = File::create("tests/reports/report-required.xml").unwrap();
    PubNubWorld::cucumber()
        .max_concurrent_scenarios(1) // sequential execution because tomato waits for a specific request at a time for which a script is initialised.
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
        .with_writer(
            writer::Basic::stdout()
                .summarized()
                .tee(writer::JUnit::new(file, 0))
                .normalized(),
        )
        .fail_on_skipped()
        .filter_run("tests/features", move |feature, _, scenario| {
            // Filter out features and scenario which doesn't have @featureSet
            // and @contract tags.
            let current_feature = get_feature_set(&feature.tags);
            !(is_ignored_feature_set_tag(&current_feature, &feature.tags)
                || is_ignored_scenario_tag(&current_feature, &scenario.tags))
        })
        .await;
}
