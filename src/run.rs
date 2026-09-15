//! A trait for standardizing the execution of script chains across different systems.

use crate::values::Values;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// A trait for standardizing the execution of script chains across different systems.
pub trait Run {
    type Script;
    type Error;
    
    /// Runs a script on the current system, changing its state
    /// # Arguments
    /// - `script` — the script to run
    /// - `data` — parameters passed to the script
    fn run(
        &mut self,
        script: Self::Script,
        data: HashMap<String, JsonValue>,
    ) -> Result<Values, Self::Error>;
}