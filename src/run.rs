use crate::values::Values;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub trait RunScript {
    type Script;
    type Error;

    fn init_script(
        &self,
        script: Self::Script,
        data: HashMap<String, JsonValue>,
    );

    fn run(
        &self,
        script: Self::Script,
        data: HashMap<String, JsonValue>,
    ) -> Result<Values, Self::Error>;

    fn clean() -> Result<(), Self::Error>;

    fn reboot() -> Result<(), Self::Error>;
}
