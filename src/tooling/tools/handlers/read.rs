use crate::tooling::spec::{BoxFuture, ToolHandlerArgument};

pub fn read_handler(args: ToolHandlerArgument) -> BoxFuture<'static, serde_json::Value> {
    Box::pin(async move {
        args.get("file_path")
            .and_then(|v| v.as_str())
            .map(|file_path| {
                let result = std::fs::read_to_string(file_path);
                match result {
                    Ok(content) => serde_json::Value::String(content),
                    Err(e) => serde_json::Value::String(format!("Error reading file: {}", e)),
                }
            })
            .unwrap_or_else(|| serde_json::Value::String("Missing 'file_path' argument".to_string()))
    })
}   