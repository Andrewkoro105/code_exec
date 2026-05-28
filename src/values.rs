use std::collections::HashMap;
use serde_json::Value as JsonValue;

pub struct Values {
    json_values: HashMap<String, JsonValue>,
    result_name: String,
}

impl Values {
    pub fn new(json_values: HashMap<String, JsonValue>, result_name: String) -> Self {
        Self {
            json_values,
            result_name,
        }
    }

    pub fn get_values(&self) -> &HashMap<String, JsonValue> {
        &self.json_values
    }

    pub fn get_result(&self) -> JsonValue {
        self.json_values
            .get(&self.result_name)
            .cloned()
            .unwrap_or(JsonValue::Null)
    }
}