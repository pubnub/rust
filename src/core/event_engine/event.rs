/// Event engine external event.
///
/// State machine uses events to calculate transition path and list of effects
/// invocations.
///
/// Types which are expected to be used as events should implement this trait.  
pub trait Event {
    /// Event identifier.
    fn id(&self) -> &str;
}
