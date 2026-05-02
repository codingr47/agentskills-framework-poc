use crate::tooling::spec::{BoxFuture, ToolHandlerArgument};
use std::process::Command;

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

                let output = cmd.output().expect("Failed to execute command");

                if !output.stdout.is_empty() {
                    print!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                    serde_json::Value::String(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    print!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                    serde_json::Value::String(String::from_utf8_lossy(&output.stderr).to_string())
                }
            })
            .unwrap_or_else(|| serde_json::Value::String("Missing 'command' argument".to_string()))
    })
}
