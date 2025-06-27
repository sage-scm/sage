pub mod commit;
pub mod prompts;

use anyhow::{anyhow, Context, Result};
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{self, ChatCompletionRequest},
};
use std::env;

/// Asks the AI with a prompt
pub async fn ask(prompt: &str) -> Result<String> {
    let config = sage_config::ConfigManager::new()?;
    let cfg = config.load()?;

    let api_url = cfg.ai.api_url;
    let mut api_key = cfg.ai.api_key;
    let ai_model = cfg.ai.model;

    // Get API key - for local endpoints like Ollama, we can use a dummy key
    if api_key.is_empty() {
        // Check if it's a local endpoint that doesn't need authentication
        if api_url.contains("localhost") || api_url.contains("127.0.0.1") {
            api_key = String::from("ollama"); // Dummy key for local endpoints
        } else {
            api_key = env::var("OPENAI_API_KEY")
                .context("Failed to get OPENAI_API_KEY environment variable")?;
        }
    }

    // Build client
    let mut client = OpenAIClient::builder()
        .with_endpoint(api_url)
        .with_api_key(&api_key)
        .build()
        .expect("Failed to build OpenAI client");

    // Create request with the o4-mini model for speed
    let req = ChatCompletionRequest::new(
        ai_model, // Using o4-mini for speed
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from(prompt)),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
    );

    // Note: Performance optimization parameters removed due to API compatibility issues
    // The o4-mini model should still be faster than larger models

    // Get response
    let result = client
        .chat_completion(req)
        .await
        .context("Failed to get chat completion")?;

    // Ensure we have choices
    if result.choices.is_empty() {
        return Err(anyhow!("No choices returned from API"));
    }

    // Extract and return content
    match &result.choices[0].message.content {
        Some(content) => Ok(content.to_string()),
        None => Err(anyhow!("No content in the response message")),
    }
}
