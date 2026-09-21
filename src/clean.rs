/// Clearing the System Status
pub trait Clean {
    type Script;
    type Error;
    /// Clearing the System Status
    fn clean(&mut self) -> impl Future<Output = Result<(), Self::Error>>;
}
