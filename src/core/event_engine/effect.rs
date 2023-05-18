pub trait Effect {
    /// Unique effect identifier.
    fn id(&self) -> String;

    /// Run work associated with effect.
    fn run<F>(&self, f: F)
    where
        F: Fn();

    /// Cancel any ongoing effect's work.
    fn cancel(&self);
}
