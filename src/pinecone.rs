use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PineconeError {
    #[error("HTTP request error: {0}")]
    Request(#[from] ReqwestError),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("API error: {resource} not found")]
    NotFound { resource: String },

    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct PineconeClient {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct AssistantContext {
    pub query: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct AssistantContextResponse {
    pub snippets: Vec<serde_json::Value>,
    pub usage: serde_json::Value,
}

impl PineconeClient {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    pub async fn assistant_context(
        &self,
        assistant_name: &str,
        query: &str,
        top_k: Option<u32>,
    ) -> Result<AssistantContextResponse, PineconeError> {
        let url = format!(
            "{}/assistant/chat/{}/context",
            self.base_url, assistant_name
        );

        let request_body = AssistantContext {
            query: query.to_string(),
            top_k,
        };

        let response = self
            .client
            .post(&url)
            .header("Api-Key", &self.api_key)
            .header("accept", "application/json")
            .header("Content-Type", "application/json")
            .header("X-Pinecone-API-Version", "2025-04")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            match status.as_u16() {
                404 => {
                    return Err(PineconeError::NotFound {
                        resource: format!("assistant \"{assistant_name}\""),
                    });
                }
                s => {
                    return Err(PineconeError::Api {
                        status: s,
                        message: error_text,
                    });
                }
            }
        }

        Ok(response.json::<AssistantContextResponse>().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_somke() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/assistant/chat/test-assistant/context")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"snippets": [{"text": "snippet 1"}, {"text": "snippet 2"}], "usage": {"total_tokens": 100}}"#)
            .create();

        let client = PineconeClient::new("test-api-key".to_string(), server.url());

        let result = client
            .assistant_context("test-assistant", "test query", None)
            .await;

        mock.assert();
        let response = result.unwrap();
        assert_eq!(response.snippets[0]["text"], "snippet 1");
        assert_eq!(response.snippets[1]["text"], "snippet 2");
    }

    #[tokio::test]
    async fn test_query_assistant_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/assistant/chat/test-assistant/context")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create();

        let client = PineconeClient::new("invalid-api-key".to_string(), server.url());

        let result = client
            .assistant_context("test-assistant", "test query", None)
            .await;

        mock.assert();
        assert!(result.is_err());
        match result {
            Err(PineconeError::Api { status, .. }) => assert_eq!(status, 401),
            _ => panic!("Expected API error"),
        }
    }
}
