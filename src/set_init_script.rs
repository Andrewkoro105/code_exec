//! A trait for standardizing the execution of script chains across different systems.

use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// A trait for standardizing the configuration of the system's initial state. This state will not be reset when [`clean()`](clean::Clean::clean) is called.
pub trait SetInitScript {
    type Script;
    type Error;

    /// Sets the initial state of the system.
    /// # Arguments
    /// - `script` - a script to restore the system to its initial state
    /// - `data` - parameters passed to the script
    fn set_init_script(&mut self, script: Self::Script, data: HashMap<String, JsonValue>) -> Result<(), Self::Error>;
}