use dotenv::{dotenv};
use async_openai::{Client, config::OpenAIConfig};
use serde_json::{Value, json};
use std::{env, process};
use pocagentskills::tooling::tools::{ToolsManager};
use pocagentskills::utils::system_prompts::get_system_prompt;
use pocagentskills::ui::chat::run_chat_ui;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let tools_manager = ToolsManager::new();
    let tools_specifications = tools_manager.json();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let model_name = env::var("OPENROUTER_MODEL").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_MODEL is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);
    let mut messages: Vec<serde_json::Value> = vec![];
    let system_prompt = get_system_prompt();
    run_chat_ui()?;
    messages.push(json!({
        "role": "system",
        "content": system_prompt
    }));

    // messages.push(json!({
    //     "role": "user",
    //     "content": args.prompt
    // }));
    print!("-------------------SYS PROMPT: {}-------------------\n\n\n", system_prompt);
    loop {
        #[allow(unused_variables)]
        let response: Value = client
            .chat()
            .create_byot(json!({
                "messages": messages,
                "model": model_name,
                "tools": tools_specifications
            }))
            .await?;

        let assistant_message = response["choices"][0]["message"].clone();
        messages.push(assistant_message.clone());


        eprintln!("Logs from your program will appear here!");
        
        if let Some(tool_calls) = assistant_message["tool_calls"].as_array() {
            for tool_call in tool_calls {
                if let Some(tool_call_function) = tool_call["function"].as_object() {
                    let tool_name = tool_call_function["name"].as_str().unwrap().to_string();
                    let arguments = tool_call_function["arguments"].as_str().unwrap();
                    let output = tools_manager.execute(tool_name, serde_json::from_str(arguments).unwrap()).await;
                    if let Some(output_value) = output {
                       messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_call["id"].as_str().unwrap(),
                            "content": output_value
                       }));
                    }
                }
            }           
        } else {
            if let Some(content) = assistant_message["content"].as_str() {
                if !content.is_empty() {
                    println!("{}", content);
                }
            }
            break;
        }
    }
    

    Ok(())
}
