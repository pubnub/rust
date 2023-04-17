//! Access manager builders module.
//!
//! This module contains all builders for the PAM management operations.

#[doc(inline)]
pub use grant_token::{GrantTokenRequest, GrantTokenRequestBuilder};

#[cfg(not(feature = "serde"))]
#[doc(inline)]
pub use grant_token::{
    GrantTokenRequestWithDeserializerBuilder, GrantTokenRequestWithSerializerBuilder,
};
pub mod grant_token;

#[cfg(not(feature = "serde"))]
#[doc(inline)]
pub use revoke::RevokeTokenRequestWithDeserializerBuilder;
#[doc(inline)]
pub use revoke::{RevokeTokenRequest, RevokeTokenRequestBuilder};
pub mod revoke;
