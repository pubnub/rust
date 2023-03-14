use serde::{Serialize, Deserialize};

impl JsonParser for Parser<T> {
  async fn decode(&self, input: String) {
    // serde_json::from_str(input).unwrap();
  }

  async fn encode(&self, input: T) {
    // serde_json::to_string(&input).unwrap();
  }
}