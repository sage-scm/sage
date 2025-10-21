use anyhow::{Result, anyhow};
use rig::{client::CompletionClient, completion::Prompt};

mod commit;
mod context;
mod prompts;

use context::{ai_context, AiProvider};

pub use commit::commit_message;

/// Asks the AI with a prompt
pub async fn ask(prompt: &str) -> Result<String> {
    let context = ai_context()?;

    // Build agent for OpenAI; Ollama uses a direct HTTP call
    let agent = match &context.provider {
        AiProvider::OpenAI { client } => {
            let mut agent_builder = client.agent(&context.model);
            if let Some(max_tokens) = context.max_tokens {
                agent_builder = agent_builder.max_tokens(max_tokens);
            }
            Some(agent_builder.build())
        }
        AiProvider::Ollama { .. } => None,
    };

    // Retry logic
    let mut attempts = context.max_retries;
    let mut last_error = None;
    while attempts > 0 {
        match &context.provider {
            AiProvider::OpenAI { .. } => {
                let agent = agent.as_ref().expect("agent must exist for OpenAI");
                match tokio::time::timeout(context.timeout, agent.prompt(prompt)).await {
                    Ok(Ok(response)) => {
                        let content = response.to_string();
                        if content.trim().is_empty() {
                            last_error = Some(anyhow!("AI provider returned empty response"));
                        } else {
                            return Ok(content);
                        }
                    }
                    Ok(Err(e)) => {
                        last_error = Some(anyhow!("AI request failed: {}", e));
                    }
                    Err(_) => {
                        last_error = Some(anyhow!(
                            "Request timed out after {} seconds",
                            context.timeout.as_secs()
                        ));
                    }
                }
            }
            AiProvider::Ollama { http, base_url } => {
                // POST /api/chat with model + messages; stream disabled
                // Minimal payload sufficient for Ollama
                #[derive(serde::Serialize)]
                struct ChatRequest<'a> {
                    model: &'a str,
                    messages: Vec<Message<'a>>,
                    stream: bool,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    options: Option<Options>,
                }

                #[derive(serde::Serialize)]
                struct Message<'a> {
                    role: &'a str,
                    content: &'a str,
                }

                #[derive(serde::Deserialize)]
                struct ChatResponse {
                    message: Option<OMessage>,
                    // Some models might return `response` in non-chat endpoints
                    response: Option<String>,
                    #[serde(default)]
                    error: Option<String>,
                }

                #[derive(serde::Deserialize)]
                struct OMessage {
                    content: String,
                }

                #[derive(serde::Serialize)]
                struct Options {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    #[serde(rename = "num_predict")]
                    num_predict: Option<u64>,
                }

                // Build endpoint robustly: support base URLs with or without "/api"
                fn build_endpoint(base: &str, path: &str) -> String {
                    let base = base.trim_end_matches('/');
                    if base.ends_with("/api") {
                        format!("{}/{}", base, path.trim_start_matches('/'))
                    } else {
                        format!("{}/api/{}", base, path.trim_start_matches('/'))
                    }
                }

                let chat_url = build_endpoint(base_url, "chat");
                let body = ChatRequest {
                    model: &context.model,
                    messages: vec![Message { role: "user", content: prompt }],
                    stream: false,
                    options: Some(Options { num_predict: context.max_tokens }),
                };

                let fut = http.post(&chat_url).json(&body).send();
                match tokio::time::timeout(context.timeout, fut).await {
                    Ok(Ok(resp)) => {
                        if resp.status().is_success() {
                            match resp.json::<ChatResponse>().await {
                                Ok(parsed) => {
                                    if let Some(err) = parsed.error {
                                        last_error = Some(anyhow!("Ollama error: {}", err));
                                    } else {
                                        let content = parsed
                                            .message
                                            .map(|m| m.content)
                                            .or(parsed.response)
                                            .unwrap_or_default();
                                        if content.trim().is_empty() {
                                            last_error = Some(anyhow!(
                                                "AI provider returned empty response"
                                            ));
                                        } else {
                                            return Ok(content);
                                        }
                                    }
                                }
                                Err(e) => {
                                    last_error = Some(anyhow!(
                                        "Failed to parse Ollama response: {}",
                                        e
                                    ));
                                }
                            }
                        } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                            // Fallback to generate endpoint
                            #[derive(serde::Serialize)]
                            struct GenerateRequest<'a> {
                                model: &'a str,
                                prompt: &'a str,
                                stream: bool,
                                #[serde(skip_serializing_if = "Option::is_none")]
                                options: Option<Options>,
                            }

                            #[derive(serde::Deserialize)]
                            struct GenerateResponse {
                                #[serde(default)]
                                response: String,
                                #[serde(default)]
                                error: Option<String>,
                            }

                            let gen_url = build_endpoint(base_url, "generate");
                            let gen_body = GenerateRequest {
                                model: &context.model,
                                prompt,
                                stream: false,
                                options: Some(Options { num_predict: context.max_tokens }),
                            };

                            let gen_fut = http.post(&gen_url).json(&gen_body).send();
                            match tokio::time::timeout(context.timeout, gen_fut).await {
                                Ok(Ok(gen_resp)) => {
                                    if !gen_resp.status().is_success() {
                                        last_error = Some(anyhow!(
                                            "Ollama request failed: {}",
                                            gen_resp.status()
                                        ));
                                    } else {
                                        match gen_resp.json::<GenerateResponse>().await {
                                            Ok(parsed) => {
                                                if let Some(err) = parsed.error {
                                                    last_error = Some(anyhow!(
                                                        "Ollama error: {}",
                                                        err
                                                    ));
                                                } else {
                                                    let content = parsed.response;
                                                    if content.trim().is_empty() {
                                                        last_error = Some(anyhow!(
                                                            "AI provider returned empty response"
                                                        ));
                                                    } else {
                                                        return Ok(content);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                last_error = Some(anyhow!(
                                                    "Failed to parse Ollama response: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                }
                                Ok(Err(e)) => {
                                    last_error = Some(anyhow!("AI request failed: {}", e));
                                }
                                Err(_) => {
                                    last_error = Some(anyhow!(
                                        "Request timed out after {} seconds",
                                        context.timeout.as_secs()
                                    ));
                                }
                            }
                        } else {
                            last_error = Some(anyhow!("Ollama request failed: {}", resp.status()));
                        }
                    }
                    Ok(Err(e)) => {
                        last_error = Some(anyhow!("AI request failed: {}", e));
                    }
                    Err(_) => {
                        last_error = Some(anyhow!(
                            "Request timed out after {} seconds",
                            context.timeout.as_secs()
                        ));
                    }
                }
            }
        }

        attempts -= 1;
        if attempts == 0 {
            break;
        }

        if !context.retry_delay.is_zero() {
            tokio::time::sleep(context.retry_delay).await;
        }
    }

    Err(last_error
        .unwrap_or_else(|| anyhow!("Failed to get response from AI provider"))
        .context("Failed to get chat completion"))
}
