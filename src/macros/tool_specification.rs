#[macro_export]
macro_rules! tool_specification {
    ($name:ident, $description:expr, $arguments:expr) => {
        pub struct $name {
            pub name: String,
            pub description: String,
            pub parameters: FunctionParametersObject
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    name: stringify!($name).to_string(),
                    description: $description.to_string(),
                    parameters: $arguments,
                }
            }

            pub fn json(&self) -> serde_json::Value {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": self.name,
                        "description": self.description,
                        "parameters": self.parameters.json()
                    }
                })
            }
        }

        impl crate::tooling::spec::LLMTool for $name {
            fn json(&self) -> serde_json::Value {
                self.json()
            }
        }
    };
}
