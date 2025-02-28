use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub pinecone_api_key: String,
    pub pinecone_assistant_host: String,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        const PINECONE_API_KEY: &str = "PINECONE_API_KEY";
        const PINECONE_ASSISTANT_HOST: &str = "PINECONE_ASSISTANT_HOST";
        const LOG_LEVEL: &str = "LOG_LEVEL";

        let pinecone_api_key = env::var(PINECONE_API_KEY).expect(&format!(
            "Missing environment variable: {}",
            PINECONE_API_KEY
        ));

        let pinecone_assistant_host = env::var(PINECONE_ASSISTANT_HOST)
            .unwrap_or_else(|_| "https://prod-1-data.ke.pinecone.io".to_string());

        let log_level = env::var(LOG_LEVEL).unwrap_or_else(|_| "info".to_string());

        Self {
            pinecone_api_key,
            pinecone_assistant_host,
            log_level,
        }
    }
}
