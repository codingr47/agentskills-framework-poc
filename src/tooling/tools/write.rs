use crate::{tool_specification};
use crate::tooling::spec::{ FunctionParametersObject, FunctionParametersObjectPropertiesObject, FunctionParametersObjectPropertyObject};

tool_specification!(
    Write,
    "Write content to a file",
    crate::tooling::spec::FunctionParametersObject {
        name: "write".to_string(),
        properties: crate::tooling::spec::FunctionParametersObjectPropertiesObject {
            arguments: vec![
                crate::tooling::spec::FunctionParametersObjectPropertyObject {
                    name: "file_path".to_string(),
                    description: "The path of the file to write to".to_string(),
                    r#type: "string".to_string(),
                },
                crate::tooling::spec::FunctionParametersObjectPropertyObject {
                    name: "content".to_string(),
                    description: "The content to write to the file".to_string(),
                    r#type: "string".to_string(),
                },
            ],
        },
        required: vec!["file_path".to_string(), "content".to_string()],
    }
);