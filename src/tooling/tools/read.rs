use crate::{tool_specification};
use crate::tooling::spec::{ FunctionParametersObject, FunctionParametersObjectPropertiesObject, FunctionParametersObjectPropertyObject};

tool_specification!(
    Read,
    "Read and return the contents of a file.",
    crate::tooling::spec::FunctionParametersObject {
        name: "file_path".to_string(),
        properties: crate::tooling::spec::FunctionParametersObjectPropertiesObject {
            arguments: vec![
                crate::tooling::spec::FunctionParametersObjectPropertyObject {
                    name: "file_path".to_string(),
                    description: "The path to the file to read".to_string(),
                    r#type: "string".to_string(),
                },
            ],
        },
        required: vec!["file_path".to_string()],
    }
);