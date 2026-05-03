use crate::tool_specification;
use crate::tooling::spec::{
    FunctionParametersObject
};

tool_specification!(
    ReadSkill,
    "Read a specific skill and return its output",
    crate::tooling::spec::FunctionParametersObject {
        name: "ReadSkill".to_string(),
        properties: crate::tooling::spec::FunctionParametersObjectPropertiesObject {
            arguments: vec![
                crate::tooling::spec::FunctionParametersObjectPropertyObject {
                    name: "skill_name".to_string(),
                    description: "The name of the skill to be read".to_string(),
                    r#type: "string".to_string(),
                },
                crate::tooling::spec::FunctionParametersObjectPropertyObject {
                    name: "reference_path".to_string(),
                    description:
                        "Optional reference path within the skill to read for incremental reading"
                            .to_string(),
                    r#type: "string".to_string(),
                }
            ],
        },
        required: vec!["skill_name".to_string()],
    }
);
