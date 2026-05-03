use std::{collections::BTreeMap, sync::mpsc::Receiver};

use async_openai::{Client, config::OpenAIConfig};
use futures_util::StreamExt;
use serde_json::{Value, json};

use crate::{
    tooling::tools::ToolsManager, ui::chat::ChatMessageHandler,
    utils::system_prompts::get_system_prompt,
};

pub type AgentResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Default)]
struct PendingToolCall {
    id: String,
    name: String,
    arguments: String,
}

struct AssistantTurn {
    content: String,
    tool_calls: Vec<PendingToolCall>,
}

pub async fn run_agent_loop(
    client: Client<OpenAIConfig>,
    model_name: String,
    tools_manager: ToolsManager,
    chat_message_handler: ChatMessageHandler,
    input_rx: Receiver<String>,
) -> AgentResult<()> {
    let tools_specifications = tools_manager.json();
    let mut messages = vec![json!({
        "role": "system",
        "content": get_system_prompt()
    })];

    while let Ok(user_message) = input_rx.recv() {
        emit_user_message(&chat_message_handler, &user_message);
        messages.push(json!({
            "role": "user",
            "content": user_message
        }));

        loop {
            let assistant_turn = stream_assistant_turn(
                &client,
                &messages,
                &model_name,
                &tools_specifications,
                &chat_message_handler,
            )
            .await?;

            let has_tool_calls = !assistant_turn.tool_calls.is_empty();
            messages.push(assistant_turn.message());

            if !has_tool_calls {
                break;
            }

            execute_tool_calls(
                &tools_manager,
                &chat_message_handler,
                &assistant_turn.tool_calls,
                &mut messages,
            )
            .await;
        }
    }

    Ok(())
}

fn emit_user_message(chat_message_handler: &ChatMessageHandler, user_message: &str) {
    let user_stream = chat_message_handler.user();
    user_stream.stream(user_message.to_string());
    user_stream.end();
}

async fn stream_assistant_turn(
    client: &Client<OpenAIConfig>,
    messages: &[Value],
    model_name: &str,
    tools_specifications: &[Value],
    chat_message_handler: &ChatMessageHandler,
) -> AgentResult<AssistantTurn> {
    let mut stream = client
        .chat()
        .create_stream_byot(json!({
            "messages": messages,
            "model": model_name,
            "tools": tools_specifications,
            "stream": true
        }))
        .await?;

    let assistant_stream = chat_message_handler.assistant();
    let mut content = String::new();
    let mut tool_calls: BTreeMap<usize, PendingToolCall> = BTreeMap::new();

    while let Some(chunk_res) = stream.next().await {
        let chunk: Value = chunk_res?;
        let delta = &chunk["choices"][0]["delta"];

        if let Some(content_delta) = delta["content"].as_str() {
            content.push_str(content_delta);
            assistant_stream.stream(content_delta.to_string());
        }

        collect_tool_call_deltas(delta, &mut tool_calls);
    }

    assistant_stream.end();

    Ok(AssistantTurn {
        content,
        tool_calls: tool_calls.into_values().collect(),
    })
}

fn collect_tool_call_deltas(delta: &Value, tool_calls: &mut BTreeMap<usize, PendingToolCall>) {
    let Some(tool_call_deltas) = delta["tool_calls"].as_array() else {
        return;
    };

    for tool_call_delta in tool_call_deltas {
        let Some(index) = tool_call_delta["index"].as_u64() else {
            continue;
        };

        tool_calls
            .entry(index as usize)
            .or_default()
            .append_delta(tool_call_delta);
    }
}

async fn execute_tool_calls(
    tools_manager: &ToolsManager,
    chat_message_handler: &ChatMessageHandler,
    tool_calls: &[PendingToolCall],
    messages: &mut Vec<Value>,
) {
    for tool_call in tool_calls {
        let arguments_value = tool_call.arguments_value();
        let tool_stream =
            chat_message_handler.tool(tool_call.name.clone(), arguments_value.clone());

        if !approve_tool_call(chat_message_handler, tool_call, &arguments_value) {
            tool_stream.stream(" [x]".to_string());
            tool_stream.end();
            messages.push(json!({
                 "role": "tool",
                 "tool_call_id": tool_call.id,
                 "content": "Tool execution denied by user"
            }));
            continue;
        }

        if let Some(output_value) = tools_manager
            .execute(tool_call.name.clone(), tool_call.arguments_object())
            .await
        {
            tool_stream.stream(" [ok]".to_string());
            tool_stream.end();
            messages.push(json!({
                 "role": "tool",
                 "tool_call_id": tool_call.id,
                 "content": tool_output_content(output_value)
            }));
        } else {
            tool_stream.stream(" [x]".to_string());
            tool_stream.end();
            messages.push(json!({
                 "role": "tool",
                 "tool_call_id": tool_call.id,
                 "content": "Tool execution failed: no matching tool handler"
            }));
        }
    }
}

fn approve_tool_call(
    chat_message_handler: &ChatMessageHandler,
    tool_call: &PendingToolCall,
    arguments_value: &Value,
) -> bool {
    chat_message_handler.request_approval(format!(
        "Execute tool `{}` with arguments {}?",
        tool_call.name, arguments_value
    ))
}

fn tool_output_content(output_value: Value) -> String {
    match output_value {
        Value::String(output) => output,
        other => other.to_string(),
    }
}

impl AssistantTurn {
    fn message(&self) -> Value {
        if self.tool_calls.is_empty() {
            return json!({
                "role": "assistant",
                "content": self.content
            });
        }

        json!({
            "role": "assistant",
            "content": if self.content.is_empty() {
                Value::Null
            } else {
                Value::String(self.content.clone())
            },
            "tool_calls": self
                .tool_calls
                .iter()
                .map(PendingToolCall::openai_message_value)
                .collect::<Vec<_>>()
        })
    }
}

impl PendingToolCall {
    fn append_delta(&mut self, delta: &Value) {
        if let Some(id) = delta["id"].as_str() {
            self.id = id.to_string();
        }

        if let Some(name) = delta["function"]["name"].as_str() {
            self.name.push_str(name);
        }

        if let Some(arguments) = delta["function"]["arguments"].as_str() {
            self.arguments.push_str(arguments);
        }
    }

    fn openai_message_value(&self) -> Value {
        json!({
            "id": self.id,
            "type": "function",
            "function": {
                "name": self.name,
                "arguments": self.arguments
            }
        })
    }

    fn arguments_value(&self) -> Value {
        serde_json::from_str(&self.arguments).unwrap_or_else(|_| {
            json!({
                "raw": self.arguments
            })
        })
    }

    fn arguments_object(&self) -> serde_json::Map<String, Value> {
        match self.arguments_value() {
            Value::Object(arguments) => arguments,
            _ => serde_json::Map::new(),
        }
    }
}
