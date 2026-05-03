use crate::tooling::spec::{BoxFuture, ToolHandlerArgument};
use std::process::{Command, Stdio};

pub fn bash(args: ToolHandlerArgument) -> BoxFuture<'static, serde_json::Value> {
    Box::pin(async move {
        args.get("command")
            .and_then(|v| v.as_str())
            .map(|command| {
                let command_parts: Vec<String> =
                    command.split(" ").map(|s| s.to_string()).collect();
                let mut cmd = Command::new(&command_parts[0]);

                for arg in &command_parts[1..] {
                    cmd.arg(arg);
                }

                let output = match cmd
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                {
                    Ok(output) => output,
                    Err(error) => {
                        return serde_json::json!({
                            "success": false,
                            "exit_code": null,
                            "stdout": "",
                            "stderr": error.to_string(),
                        });
                    }
                };

                serde_json::json!({
                    "success": output.status.success(),
                    "exit_code": output.status.code(),
                    "stdout": String::from_utf8_lossy(&output.stdout),
                    "stderr": String::from_utf8_lossy(&output.stderr),
                })
            })
            .unwrap_or_else(|| serde_json::Value::String("Missing 'command' argument".to_string()))
    })
}
