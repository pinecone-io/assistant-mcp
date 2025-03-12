use crate::config::Config;
use crate::pinecone::{PineconeClient, PineconeError};
use mcp_spec::content::Content;
use mcp_spec::handler::{PromptError, ResourceError, ToolError};
use mcp_spec::prompt::Prompt;
use mcp_spec::{protocol::ServerCapabilities, resource::Resource, tool::Tool};
use mcp_server::router::CapabilitiesBuilder;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

const TOOL_ASSISTANT_CONTEXT: &'static str = "assistant_context";

const PARAM_ASSISTANT_NAME: &'static str = "assistant_name";
const PARAM_QUERY: &'static str = "query";
const PARAM_TOP_K: &'static str = "top_k";

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("Pinecone error: {0}")]
    Pinecone(#[from] PineconeError),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
}

impl From<RouterError> for ToolError {
    fn from(err: RouterError) -> Self {
        match err {
            RouterError::Pinecone(e) => ToolError::ExecutionError(e.to_string()),
            RouterError::InvalidParameters(msg) => ToolError::InvalidParameters(msg),
        }
    }
}

#[derive(Clone)]
pub struct PineconeAssistantRouter {
    client: PineconeClient,
    tools: Vec<Tool>,
}

impl PineconeAssistantRouter {
    pub fn new(config: Config) -> Self {
        tracing::info!(
            "Creating new PineconeAssistantRouter [Host: {}]",
            config.pinecone_assistant_host
        );
        let client = PineconeClient::new(config.pinecone_api_key, config.pinecone_assistant_host);
        tracing::info!("Successfully initialized Pinecone client");
        Self {
            client,
            tools: vec![Tool::new(
                TOOL_ASSISTANT_CONTEXT.to_string(),
                "Retrieve context snippets from a Pinecone assistant knowledge base".to_string(),
                serde_json::json!({
                "type": "object",
                "properties": {
                    PARAM_ASSISTANT_NAME: {
                        "type": "string",
                        "description": "Name of the Pinecone assistant"
                    },
                    PARAM_QUERY: {
                        "type": "string",
                        "description": "The query to use for generating context."
                    },
                    PARAM_TOP_K: {
                        "type": "integer",
                        "description": "Number of context snippets to return. Default is 15."
                        }
                    },
                    "required": [PARAM_ASSISTANT_NAME, PARAM_QUERY]
                }),
            )],
        }
    }

    async fn handle_assistant_context(
        &self,
        arguments: Value,
    ) -> Result<Vec<Content>, RouterError> {
        tracing::debug!("Processing {TOOL_ASSISTANT_CONTEXT} arguments");
        let assistant_name = arguments[PARAM_ASSISTANT_NAME].as_str().ok_or_else(|| {
            RouterError::InvalidParameters(format!("{} must be a string", PARAM_ASSISTANT_NAME))
        })?;
        let query = arguments[PARAM_QUERY].as_str().ok_or_else(|| {
            RouterError::InvalidParameters(format!("{} must be a string", PARAM_QUERY))
        })?;
        let top_k = arguments[PARAM_TOP_K].as_u64().map(|v| v as u32);

        tracing::info!(
            "Making request to Pinecone API for assistant: {} with top_k: {:?}",
            assistant_name,
            top_k
        );

        let response = self
            .client
            .assistant_context(assistant_name, query, top_k)
            .await?;

        tracing::info!("Successfully received response from Pinecone API");
        Ok(vec![Content::text(response.snippets.to_string())])
    }
}

impl mcp_server::Router for PineconeAssistantRouter {
    fn name(&self) -> String {
        "pinecone-assistant".to_string()
    }

    fn instructions(&self) -> String {
        format!(
            "This server provides tools to interact with Pinecone's Assistant API. \
        The {TOOL_ASSISTANT_CONTEXT} tool allows you to retrieve relevant context snippets from given knowledge base assistants. \
        Returned value structure: \
        An array of relevant document snippets, each containing: \
          - content: The text content of the snippet \
          - score: A relevance score (float) \
          - reference: Source information including document ID and location \
        You can tune the number of returned snippets by setting the `top_k` parameter."
        )
    }

    fn capabilities(&self) -> ServerCapabilities {
        tracing::debug!("Building server capabilities");
        CapabilitiesBuilder::new().with_tools(true).build()
    }

    fn list_tools(&self) -> Vec<Tool> {
        tracing::debug!("Listing available tools");
        self.tools.clone()
    }

    fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Content>, ToolError>> + Send + 'static>> {
        tracing::info!("Calling tool: {}", tool_name);
        let router = self.clone();
        match tool_name {
            TOOL_ASSISTANT_CONTEXT => Box::pin(async move {
                router
                    .handle_assistant_context(arguments)
                    .await
                    .map_err(Into::into)
            }),
            _ => {
                tracing::error!("Tool not found: {}", tool_name);
                let tool_name = tool_name.to_string();
                Box::pin(async move {
                    Err(ToolError::NotFound(format!("Tool {} not found", tool_name)))
                })
            }
        }
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![]
    }

    fn read_resource(
        &self,
        _uri: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, ResourceError>> + Send + 'static>> {
        Box::pin(async {
            Err(ResourceError::NotFound(
                "No resources available".to_string(),
            ))
        })
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        vec![]
    }

    fn get_prompt(
        &self,
        prompt_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, PromptError>> + Send + 'static>> {
        let prompt_name = prompt_name.to_string();
        Box::pin(async move {
            Err(PromptError::NotFound(format!(
                "Prompt {} not found",
                prompt_name
            )))
        })
    }
}
