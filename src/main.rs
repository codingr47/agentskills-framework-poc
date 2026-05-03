use async_openai::{Client, config::OpenAIConfig};
use dotenv::dotenv;
use pocagentskills::{
    agent::{AgentResult, run_agent_loop},
    tooling::tools::ToolsManager,
    ui::chat::{chat_message_handler, run_chat_ui},
};
use std::{env, process};

#[tokio::main]
async fn main() -> AgentResult<()> {
    dotenv().ok();

    let client = Client::with_config(openai_config());
    let model_name = env::var("OPENROUTER_MODEL").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_MODEL is not set");
        process::exit(1);
    });

    let tools_manager = ToolsManager::new();
    let (chat_message_handler, chat_receiver) = chat_message_handler();
    let (input_tx, input_rx) = std::sync::mpsc::channel::<String>();

    let ui_thread = std::thread::spawn(move || {
        run_chat_ui(chat_receiver, move |message| {
            let _ = input_tx.send(message);
        })
    });

    run_agent_loop(
        client,
        model_name,
        tools_manager,
        chat_message_handler,
        input_rx,
    )
    .await?;

    ui_thread
        .join()
        .map_err(|_| "UI thread panicked")?
        .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error })?;

    Ok(())
}

fn openai_config() -> OpenAIConfig {
    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key)
}
