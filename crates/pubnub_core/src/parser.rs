#[async_trait::async_trait]
pub trait Parser<T> {
    async fn decode(&self, input: String) -> Result<T>;
    async fn encode(&self, input: T) -> Result<String>;
}

