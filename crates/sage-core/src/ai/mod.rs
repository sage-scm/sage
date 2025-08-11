pub mod commit;
pub mod prompts;

use anyhow::{Context, Result, anyhow};
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{self, ChatCompletionRequest},
};
use std::env;
use std::time::Duration;

/// Asks the AI with a prompt
pub async fn ask(prompt: &str) -> Result<String> {
    let config = sage_config::ConfigManager::new()?;
    let cfg = config.load()?;

    let api_url = cfg.ai.api_url;
    let mut api_key = cfg.ai.api_key;
    let ai_model = cfg.ai.model;

    if api_key.is_empty() {
        api_key = if api_url.contains("localhost") || api_url.contains("127.0.0.1") {
            String::from("ollama")
        } else {
            env::var("OPENAI_API_KEY")
                .context("Failed to get OPENAI_API_KEY environment variable")?
        };
    }
    println!("AI API URL: {api_url}");
    println!("AI API KEY: {api_key}");
    println!("AI MODEL: {ai_model}");

    // Build client
    let mut client = OpenAIClient::builder()
        .with_endpoint(api_url)
        .with_api_key(&api_key)
        .build()
        .expect("Failed to build OpenAI client");

    // Retry logic
    let mut attempts = 3;
    let mut last_error = None;
    while attempts > 0 {
        let req = ChatCompletionRequest::new(
            ai_model.clone(),
            vec![chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(String::from(prompt)),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
        );

        match tokio::time::timeout(Duration::from_secs(10), client.chat_completion(req)).await {
            Ok(Ok(result)) => {
                if result.choices.is_empty() {
                    return Err(anyhow!("No choices returned from API"));
                }
                let content = result.choices[0]
                    .message
                    .content
                    .as_ref()
                    .ok_or_else(|| anyhow!("No content in response"))?;

                if content.trim().is_empty() {
                    return Err(anyhow!("Empty response content"));
                }

                return Ok(content.to_string());
            }
            Ok(Err(e)) => {
                last_error = Some(anyhow::Error::from(e));
                attempts -= 1;
                if attempts == 0 {
                    return Err(last_error.unwrap().context("Failed to get chat completion"));
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(_) => {
                last_error = Some(anyhow!("Request timed out after 10 seconds"));
                attempts -= 1;
                if attempts == 0 {
                    return Err(last_error.unwrap());
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    Err(last_error.unwrap().context("Failed to get chat completion"))
}
