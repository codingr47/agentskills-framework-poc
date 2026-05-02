use std::{collections::HashMap, pin::Pin};

pub trait LLMTool {
    fn json(&self) -> serde_json::Value;
}

pub struct FunctionParametersObject {
    pub name: String,
    pub properties: FunctionParametersObjectPropertiesObject,
    pub required: Vec<String>,
}

pub struct FunctionParametersObjectPropertyObject {
    pub name: String,
    pub r#type: String,
    pub description: String,
}

pub struct FunctionParametersObjectPropertiesObject {
    pub arguments: Vec<FunctionParametersObjectPropertyObject>,
}

pub type ToolHandlerArgument = serde_json::Map<String, serde_json::Value>;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub type LLMToolHandler =
    Box<dyn Fn(ToolHandlerArgument) -> BoxFuture<'static, serde_json::Value> + Send + Sync>;

impl FunctionParametersObject {
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": self.properties.json(),
            "required": self.required
        })
    }
}

impl FunctionParametersObjectPropertiesObject {
    pub fn json(&self) -> serde_json::Value {
        let mut map: HashMap<String, serde_json::Value> = HashMap::new();
        for i in 0..self.arguments.len() {
            let arg = &self.arguments[i];
            map.insert(arg.name.clone(), arg.json());
        }

        serde_json::Value::Object(map.into_iter().collect())
    }
}

impl FunctionParametersObjectPropertyObject {
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": self.r#type,
            "description": self.description
        })
    }
}
