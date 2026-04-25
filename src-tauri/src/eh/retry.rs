use std::time::Duration;

const MAX_RETRIES: u32 = 5;
const BASE_WAIT_SECS: u64 = 30;

/// Exponential backoff retry. Waits 30s, 60s, 120s, 240s, 480s.
/// Only retries on IP_BANNED errors (HTTP 403).
pub async fn retry_with_backoff<F, Fut, T>(mut f: F) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if e == "IP_BANNED" => {
                let wait = BASE_WAIT_SECS * 2u64.pow(attempt);
                log::warn!("IP banned, waiting {}s before retry {}", wait, attempt + 1);
                tokio::time::sleep(Duration::from_secs(wait)).await;
                attempt += 1;
                if attempt >= MAX_RETRIES {
                    return Err("IP_BANNED".into());
                }
            }
            Err(e) => return Err(e),
        }
    }
}