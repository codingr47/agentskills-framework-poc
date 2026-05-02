use crate::tooling::spec::{BoxFuture, ToolHandlerArgument};

pub fn write_handler(args: ToolHandlerArgument) -> BoxFuture<'static, serde_json::Value> {
    Box::pin(async move {
        args.get("file_path")
            .and_then(|v| v.as_str())
            .map(|file_path| {
                args.get("content")
                    .and_then(|v| v.as_str())
                    .map(|content| {
                        let result = std::fs::write(file_path, content);
                        match result {
                            Ok(_) => serde_json::Value::String(format!(
                                "Successfully wrote to file: {}",
                                file_path
                            )),
                            Err(e) => {
                                serde_json::Value::String(format!("Error writing to file: {}", e))
                            }
                        }
                    })
                    .unwrap_or_else(|| {
                        serde_json::Value::String("Missing 'content' argument".to_string())
                    })
            })
            .unwrap_or_else(|| {
                serde_json::Value::String("Missing 'file_path' argument".to_string())
            })
    })
}
