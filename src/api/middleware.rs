use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::IntoResponse,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Adds a unique request ID to each request and response.
pub async fn request_id(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();

    request.extensions_mut().insert(request_id.clone());

    let mut response = next.run(request).await;

    response
        .headers_mut()
        .insert("X-Request-Id", HeaderValue::from_str(&request_id).unwrap());

    response
}

pub struct RateLimiter {
    requests_per_second: u32,
    burst: u32,
    last_cleanup: Mutex<Instant>,
    entries: RwLock<HashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    /// Creates a new RateLimiter.
    pub fn new(requests_per_second: u32, burst: u32) -> Self {
        Self {
            requests_per_second,
            burst,
            last_cleanup: Mutex::new(Instant::now()),
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Checks if a request is within rate limits for the given key.
    pub async fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut entries = self.entries.write().await;

        if now.duration_since(*self.last_cleanup.lock().await) > Duration::from_secs(60) {
            entries.retain(|_, times| {
                times.retain(|&t| now.duration_since(t) < Duration::from_secs(1));
                !times.is_empty()
            });
            *self.last_cleanup.lock().await = now;
        }

        let bucket = entries.entry(key.to_string()).or_insert_with(Vec::new);

        bucket.retain(|&t| now.duration_since(t) < Duration::from_secs(1));

        if bucket.len() < self.requests_per_second as usize {
            bucket.push(now);
            return true;
        }

        if bucket.len() < self.burst as usize {
            let oldest = bucket.first();
            if let Some(oldest) = oldest
                && now.duration_since(*oldest) > Duration::from_secs(1)
            {
                bucket.push(now);
                return true;
            }
        }

        false
    }
}

pub type RateLimiterType = Arc<RateLimiter>;

/// Creates a new rate limiter.
pub fn create_rate_limiter(requests_per_second: u32, burst: u32) -> RateLimiterType {
    Arc::new(RateLimiter::new(requests_per_second, burst))
}

/// Rate limiting middleware.
pub async fn rate_limit(request: Request, next: Next) -> Response {
    let ip = extract_client_ip(&request);

    let rate_limiter: Option<&Arc<RateLimiter>> = request.extensions().get();

    if let Some(rate_limiter) = rate_limiter
        && !rate_limiter.check(&ip).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("X-RateLimit-Limit", "rate limit")],
            "Too many requests",
        )
            .into_response();
    }

    next.run(request).await
}

fn extract_client_ip(request: &Request) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "127.0.0.1".to_string())
}
