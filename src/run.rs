//! A trait for standardizing the execution of script chains across different systems.

use crate::values::Values;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// A trait for standardizing the execution of script chains across different systems.
pub trait RunScript {
    type Script;
    type Error;

    /// Sets the initial state of the system. This state will not be reset when [`clean()`](Self::clean) is called.
    /// # Arguments
    /// - `script` - a script to restore the system to its initial state
    /// - `data` - parameters passed to the script
    fn set_init_script(&mut self, script: Self::Script, data: HashMap<String, JsonValue>);

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

/// Clearing the System Status
pub trait Clean {
    type Script;
    type Error;
    /// Clearing the System Status
    fn clean(&mut self) -> Result<(), Self::Error>;
}

/// A complete system reboot
pub trait Reboot {
    type Script;
    type Error;
    /// A complete system reboot
    fn reboot(&mut self) -> Result<(), Self::Error>;
}
