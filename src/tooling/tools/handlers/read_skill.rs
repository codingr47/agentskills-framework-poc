use crate::tooling::spec::{BoxFuture, ToolHandlerArgument};
use std::{env::current_dir, path::PathBuf};

pub fn read_skill_handler(args: ToolHandlerArgument) -> BoxFuture<'static, serde_json::Value> {
    Box::pin(async move {
        args.get("skill_name")
            .and_then(|v| v.as_str())
            .map(|file_path| {
                let cwd = current_dir().expect("Couldnt get CWD");
                let cwd_path = cwd.to_str().expect("Path is invalid");
                let reference_path = args.get("reference_path");

                let mut path_to_skill = PathBuf::from(cwd_path);
                path_to_skill.push(".skills");
                path_to_skill.push(file_path);
                if let Some(reference) = reference_path {
                    path_to_skill.push("references");
                    path_to_skill.push(reference.to_string());
                } else {
                    path_to_skill.push("SKILL.md");
                }
                let result = std::fs::read_to_string(path_to_skill);
                match result {
                    Ok(content) => serde_json::Value::String(content),
                    Err(e) => serde_json::Value::String(format!("Error reading file: {}", e)),
                }
            })
            .unwrap_or_else(|| {
                serde_json::Value::String("Missing 'skill_name' argument".to_string())
            })
    })
}
