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
        .run("tests/features")
        .await;
}
