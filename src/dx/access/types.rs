//! # PAMv3 types module
//!
//! The module contains [`MetaValue`] type for token grant operation support.

/// Enum for values associated with token.
///
/// Token can be associated with flat HashMap which represent `meta`
/// information.
pub enum MetaValue {
    /// `String` value.
    String(String),

    /// `Integer` value.
    Integer(i64),

    /// `Float` / `double` value.
    Float(f64),

    /// `Boolean` value.
    Boolean(bool),

    /// `null` value.
    Null,
}

impl From<String> for MetaValue {
    fn from(value: String) -> Self {
        MetaValue::String(value)
    }
}

impl From<&str> for MetaValue {
    fn from(value: &str) -> Self {
        MetaValue::String(String::from(value))
    }
}

impl From<i64> for MetaValue {
    fn from(value: i64) -> Self {
        MetaValue::Integer(value)
    }
}

impl From<i32> for MetaValue {
    fn from(value: i32) -> Self {
        MetaValue::Integer(value.into())
    }
}

impl From<f64> for MetaValue {
    fn from(value: f64) -> Self {
        MetaValue::Float(value)
    }
}

impl From<f32> for MetaValue {
    fn from(value: f32) -> Self {
        MetaValue::Float(value.into())
    }
}

impl From<bool> for MetaValue {
    fn from(value: bool) -> Self {
        MetaValue::Boolean(value)
    }
}

impl From<()> for MetaValue {
    fn from(_: ()) -> Self {
        MetaValue::Null
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for MetaValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MetaValue::String(val) => serializer.serialize_str(val),
            MetaValue::Integer(val) => serializer.serialize_i64(*val),
            MetaValue::Float(val) => serializer.serialize_f64(*val),
            MetaValue::Boolean(val) => serializer.serialize_bool(*val),
            MetaValue::Null => serializer.serialize_unit(),
        }
    }
}
