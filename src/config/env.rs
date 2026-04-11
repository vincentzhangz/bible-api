use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub api_host: String,
    pub api_port: u16,
    pub data_dir: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/bible_api".to_string());

        let api_host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let api_port: u16 = env::var("API_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let data_dir = env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Default to data/ relative to the crate root
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
            });

        Self {
            database_url,
            api_host,
            api_port,
            data_dir,
        }
    }
}
