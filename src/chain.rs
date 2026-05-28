use crate::values::Values;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub trait Chain {
    type Script;
    type Error;

    fn chain(
        &self,
        script: Vec<Self::Script>,
        data: HashMap<String, JsonValue>,
    ) -> Result<Vec<Values>, Self::Error>;
}
