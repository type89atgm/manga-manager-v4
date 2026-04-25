use crate::eh::client::EhClient;
use crate::db::models::EhGalleryMetadata;
use std::time::Duration;

const BATCH_SIZE: usize = 25;
const BASE_DELAY_MS: u64 = 1500;

/// Batch fetch with rate limiting. 25 galleries per request, 1.5s between batches.
pub async fn batch_fetch_with(
    client: &EhClient,
    gidlist: &[(i64, String)],
    cookies: &str,
) -> Result<Vec<EhGalleryMetadata>, String> {
    let mut all_results = Vec::new();
    let chunks: Vec<_> = gidlist.chunks(BATCH_SIZE).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(Duration::from_millis(BASE_DELAY_MS)).await;
        }
        match client.fetch_gdata(chunk, cookies).await {
            Ok(results) => all_results.extend(results),
            Err(e) if e == "IP_BANNED" => return Err("IP_BANNED".into()),
            Err(e) => return Err(format!("batch {} failed: {}", i, e)),
        }
    }
    Ok(all_results)
}