use crate::error::PubNubError;

pub trait Serialize {
    fn serialize(self) -> Result<Vec<u8>, PubNubError>;
}

impl<I> Serialize for I
where
    I: Into<Vec<u8>>,
{
    fn serialize(self) -> Result<Vec<u8>, PubNubError> {
        Ok(self.into())
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use test_case::test_case;

    #[test_case(vec![1, 2, 3] => vec![1, 2, 3]; "vector of bytes")]
    #[test_case("abc" => vec![97, 98, 99]; "string slice")]
    #[test_case("abc".to_string() => vec![97, 98, 99]; "string")]
    fn serialize_vector_of_bytes(input: impl Serialize) -> Vec<u8> {
        input.serialize().unwrap()
    }
}
