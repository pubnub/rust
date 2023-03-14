pub trait Parser<T> {
    fn decode(&self, input: String) -> Result<T>;
    fn encode(&self, input: T) -> Result<String>;
}

