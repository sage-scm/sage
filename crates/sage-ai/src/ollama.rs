use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct Options {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "num_predict")]
    num_predict: Option<u64>,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Options>,
}

#[derive(Deserialize)]
struct OMessage {
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    #[serde(default)]
    message: Option<OMessage>,
    #[serde(default)]
    response: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Options>,
}

#[derive(Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    response: String,
    #[serde(default)]
    error: Option<String>,
}

fn endpoint(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.ends_with("/api") {
        format!("{}/{}", base, path.trim_start_matches('/'))
    } else {
        format!("{}/api/{}", base, path.trim_start_matches('/'))
    }
}

pub async fn request(
    http: &Client,
    base_url: &str,
    model: &str,
    prompt: &str,
    max_tokens: Option<u64>,
    timeout: Duration,
) -> Result<String> {
    let chat_url = endpoint(base_url, "chat");
    let body = ChatRequest {
        model,
        messages: vec![Message { role: "user", content: prompt }],
        stream: false,
        options: Some(Options { num_predict: max_tokens }),
    };

    let fut = http.post(&chat_url).json(&body).send();
    match tokio::time::timeout(timeout, fut).await {
        Ok(Ok(resp)) => {
            if resp.status().is_success() {
                match resp.json::<ChatResponse>().await {
                    Ok(parsed) => {
                        if let Some(err) = parsed.error {
                            Err(anyhow!(err))
                        } else {
                            let content = parsed
                                .message
                                .map(|m| m.content)
                                .or(parsed.response)
                                .unwrap_or_default();
                            if content.trim().is_empty() {
                                Err(anyhow!("empty response"))
                            } else {
                                Ok(content)
                            }
                        }
                    }
                    Err(e) => Err(anyhow!("parse error: {}", e)),
                }
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                let gen_url = endpoint(base_url, "generate");
                let gen_body = GenerateRequest {
                    model,
                    prompt,
                    stream: false,
                    options: Some(Options { num_predict: max_tokens }),
                };
                let gen_fut = http.post(&gen_url).json(&gen_body).send();
                match tokio::time::timeout(timeout, gen_fut).await {
                    Ok(Ok(gen_resp)) => {
                        if !gen_resp.status().is_success() {
                            Err(anyhow!("status {}", gen_resp.status()))
                        } else {
                            match gen_resp.json::<GenerateResponse>().await {
                                Ok(parsed) => {
                                    if let Some(err) = parsed.error {
                                        Err(anyhow!(err))
                                    } else if parsed.response.trim().is_empty() {
                                        Err(anyhow!("empty response"))
                                    } else {
                                        Ok(parsed.response)
                                    }
                                }
                                Err(e) => Err(anyhow!("parse error: {}", e)),
                            }
                        }
                    }
                    Ok(Err(e)) => Err(anyhow!("request error: {}", e)),
                    Err(_) => Err(anyhow!("timeout")),
                }
            } else {
                Err(anyhow!("status {}", resp.status()))
            }
        }
        Ok(Err(e)) => Err(anyhow!("request error: {}", e)),
        Err(_) => Err(anyhow!("timeout")),
    }
}

