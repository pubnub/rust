use crate::error::PubNubError;

// TODO: What to deserialize? Do we need that for publish?
//       Should the internal deserialization be done by the transport layer or Dx.
pub trait Deserialize {
    type Output;

    fn deserialize(self) -> Result<Self::Output, PubNubError>;
}
