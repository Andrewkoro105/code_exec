use crate::values::Values;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub trait RunScript {
    type Script;
    type Error;

    fn run_script(
        &self,
        script: Self::Script,
        data: HashMap<String, JsonValue>,
    ) -> Result<Values, Self::Error>;
}
