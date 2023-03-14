use serde::{Serialize, Deserialize};

impl JsonParser for Parser<T> {
  fn decode(&self, input: String) {
    // serde_json::from_str(input).unwrap();
  }

  fn encode(&self, input: T) {
    // serde_json::to_string(&input).unwrap();
  }
}