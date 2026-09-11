//! A feature for standardizing script execution across different systems.
use crate::values::Values;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// A feature for standardizing script execution across different systems.
pub trait RunScript {
    type Script;
    type Error;

    ///  Runs a script in an isolated environment
    /// # Arguments
    /// - `script` — the script to run
    /// - `data` — parameters passed to the script
    fn run_script(
        &self,
        script: Self::Script,
        data: HashMap<String, JsonValue>,
    ) -> Result<Values, Self::Error>;
}

