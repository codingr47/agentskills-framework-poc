use crate::tool_specification;
use crate::tooling::spec::{
    FunctionParametersObject, FunctionParametersObjectPropertiesObject,
    FunctionParametersObjectPropertyObject,
};

tool_specification!(
    Bash,
    "Execute a shell command",
    crate::tooling::spec::FunctionParametersObject {
        name: "bash".to_string(),
        properties: crate::tooling::spec::FunctionParametersObjectPropertiesObject {
            arguments: vec![
                crate::tooling::spec::FunctionParametersObjectPropertyObject {
                    name: "command".to_string(),
                    description: "The command to execute".to_string(),
                    r#type: "string".to_string(),
                },
            ],
        },
        required: vec!["command".to_string()],
    }
);
