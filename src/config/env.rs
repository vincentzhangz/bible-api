use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub api_host: String,
    pub api_port: u16,
    pub data_dir: PathBuf,
    pub db_max_connections: u32,
    pub db_acquire_timeout_secs: u64,
    pub search_limit: u32,
    pub word_frequency_limit: u32,
    pub cors_allowed_origins: Vec<String>,
    pub rate_limit_per_second: u32,
    pub rate_limit_burst: u32,
}

impl AppConfig {
/// Loads application configuration from environment variables.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/bible_api".to_string());

        let api_host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let api_port: u16 = env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);

        let data_dir = env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"));

        let db_max_connections: u32 = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        let db_acquire_timeout_secs: u64 = env::var("DB_ACQUIRE_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let search_limit: u32 = env::var("SEARCH_LIMIT")
            .unwrap_or_else(|_| "50".to_string())
            .parse()
            .unwrap_or(50);

        let word_frequency_limit: u32 = env::var("WORD_FREQUENCY_LIMIT")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);

        let cors_allowed_origins: Vec<String> = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let rate_limit_per_second: u32 = env::var("RATE_LIMIT_PER_SECOND")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        let rate_limit_burst: u32 = env::var("RATE_LIMIT_BURST")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20);

        Self {
            database_url,
            api_host,
            api_port,
            data_dir,
            db_max_connections,
            db_acquire_timeout_secs,
            search_limit,
            word_frequency_limit,
            cors_allowed_origins,
            rate_limit_per_second,
            rate_limit_burst,
        }
    }
}
