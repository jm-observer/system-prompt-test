use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::mpsc;

use super::{LlmProvider, LlmRequest, LlmResponse, StreamEvent, TokenUsage};

pub struct OpenAiProvider {
    base_url: String,
    api_key: String,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(base_url: String, api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self {
            base_url,
            api_key,
            client,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, String> {
        let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));

        let body = json!({
            "model": request.model_name,
            "messages": [
                {"role": "system", "content": request.system_prompt},
                {"role": "user", "content": request.user_message}
            ],
            "max_tokens": request.max_tokens.unwrap_or(2048),
            "temperature": request.temperature.unwrap_or(0.7),
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| format!("Read body error: {}", e))?;

        if !status.is_success() {
            return Err(format!("OpenAI API error ({}): {}", status, raw));
        }

        let data: Value =
            serde_json::from_str(&raw).map_err(|e| format!("JSON parse error: {}", e))?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = &data["usage"];
        let token_usage = TokenUsage {
            prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
        };

        Ok(LlmResponse {
            content,
            token_usage,
            raw_response: raw,
        })
    }

    async fn stream(
        &self,
        request: &LlmRequest,
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<(), String> {
        let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));

        let body = json!({
            "model": request.model_name,
            "messages": [
                {"role": "system", "content": request.system_prompt},
                {"role": "user", "content": request.user_message}
            ],
            "max_tokens": request.max_tokens.unwrap_or(2048),
            "temperature": request.temperature.unwrap_or(0.7),
            "stream": true,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("OpenAI API error ({}): {}", status, body));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || !line.starts_with("data: ") {
                    continue;
                }

                let data_str = &line[6..];
                if data_str == "[DONE]" {
                    let _ = tx
                        .send(StreamEvent {
                            event_type: "done".to_string(),
                            content: None,
                            token_usage: None,
                            error: None,
                        })
                        .await;
                    return Ok(());
                }

                if let Ok(data) = serde_json::from_str::<Value>(data_str) {
                    if let Some(content) = data["choices"][0]["delta"]["content"].as_str() {
                        let _ = tx
                            .send(StreamEvent {
                                event_type: "delta".to_string(),
                                content: Some(content.to_string()),
                                token_usage: None,
                                error: None,
                            })
                            .await;
                    }
                }
            }
        }

        let _ = tx
            .send(StreamEvent {
                event_type: "done".to_string(),
                content: None,
                token_usage: None,
                error: None,
            })
            .await;

        Ok(())
    }
}
