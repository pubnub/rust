mod mvec;
mod registry;

pub(crate) mod channel;
pub(crate) mod subscribe_loop;
pub(crate) mod subscribe_loop_supervisor;

// Explicitly allow clippy::module_inception here. We just reexport everything
// from this module to list all the dependencies cleanly in a separate file.
// This nesting never appears in the API.
#[allow(clippy::module_inception)]
mod subscription;
pub use subscription::*;
