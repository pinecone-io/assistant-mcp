use crate::config::Config;
use crate::pinecone::{PineconeClient, PineconeError};
use mcp_server::router::CapabilitiesBuilder;
use mcp_spec::content::Content;
use mcp_spec::handler::{PromptError, ResourceError, ToolError};
use mcp_spec::prompt::Prompt;
use mcp_spec::{protocol::ServerCapabilities, resource::Resource, tool::Tool};
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
                "Retrieves relevant document snippets from your Pinecone Assistant knowledge base. \
                Returns an array of text snippets from the most relevant documents. \
                You can use the 'top_k' parameter to control result count (default: 15). \
                Recommended top_k: a few (5-8) for simple/narrow queries, 10-20 for complex/broad topics.".to_string(),
                serde_json::json!({
                "type": "object",
                "properties": {
                    PARAM_ASSISTANT_NAME: {
                        "type": "string",
                        "description": "Name of an existing Pinecone assistant"
                    },
                    PARAM_QUERY: {
                        "type": "string",
                        "description": "The query to retrieve context for."
                    },
                    PARAM_TOP_K: {
                        "type": "integer",
                        "description": "The number of context snippets to retrieve. Defaults to 15."
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
        Ok(response
            .snippets
            .iter()
            .map(|snippet| Content::text(snippet.to_string()))
            .collect())
    }
}

impl mcp_server::Router for PineconeAssistantRouter {
    fn name(&self) -> String {
        "pinecone-assistant".to_string()
    }

    fn instructions(&self) -> String {
        format!(
            "This server connects to an existing Pinecone Assistant,\
            a RAG system for retrieving relevant document snippets. \
            Use the {TOOL_ASSISTANT_CONTEXT} tool to access contextual information from its knowledge base"
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
