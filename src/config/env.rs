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

fn parse_env_or_warn<T: std::str::FromStr>(key: &str, default: T) -> T {
    match env::var(key) {
        Ok(val) => val.parse().unwrap_or_else(|_| {
            tracing::warn!("Invalid value for {}, using default", key);
            default
        }),
        Err(_) => default,
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/bible_api".to_string());

        let api_host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let api_port: u16 = parse_env_or_warn("API_PORT", 8080);

        let data_dir = env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"));

        let db_max_connections: u32 = parse_env_or_warn("DB_MAX_CONNECTIONS", 10);

        let db_acquire_timeout_secs: u64 = parse_env_or_warn("DB_ACQUIRE_TIMEOUT_SECS", 30);

        let search_limit: u32 = parse_env_or_warn("SEARCH_LIMIT", 50);

        let word_frequency_limit: u32 = parse_env_or_warn("WORD_FREQUENCY_LIMIT", 100);

        let cors_allowed_origins: Vec<String> = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let rate_limit_per_second: u32 = parse_env_or_warn("RATE_LIMIT_PER_SECOND", 10);

        let rate_limit_burst: u32 = parse_env_or_warn("RATE_LIMIT_BURST", 20);

        if rate_limit_burst < rate_limit_per_second {
            tracing::warn!(
                "rate_limit_burst ({}) < rate_limit_per_second ({}), burst will have no effect",
                rate_limit_burst,
                rate_limit_per_second
            );
        }

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
