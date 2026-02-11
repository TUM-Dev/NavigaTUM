use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Attempts to load a file from the local filesystem first, falling back to downloading it via HTTP.
///
/// This allows users to provide static files in the Docker container while maintaining
/// the ability to download them in development environments.
///
/// # Arguments
///   * `filename` - The name of the file to load (e.g., "api_data.json")
///   * `cdn_url` - The CDN URL to use for downloading if the file is not found locally
///
/// # Returns
/// The file contents as bytes
pub async fn load_file_or_download(filename: &str, cdn_url: &str) -> anyhow::Result<Vec<u8>> {
    // Try to load from disk first
    if let Some(bytes) = try_load_from_disk(filename).await {
        info!(filename, "Loaded file from disk");
        return Ok(bytes);
    }

    // Fall back to downloading
    info!(filename, cdn_url, "File not found on disk, downloading");
    download_file(filename, cdn_url).await
}

/// Attempts to load a file from the local filesystem.
///
/// Looks in the following locations (in order):
/// 1. `/cdn/{filename}` - production CDN mount point
/// 2. `data/output/{filename}` - relative to current working directory
/// 3. `../data/output/{filename}` - one level up (useful when running from server/ directory)
/// 4. `../../data/output/{filename}` - two levels up (useful when running tests)
///
/// # Arguments
///   * `filename` - The name of the file to load
///
/// # Returns
/// The file contents as bytes if found, None otherwise
async fn try_load_from_disk(filename: &str) -> Option<Vec<u8>> {
    let search_paths = vec![
        PathBuf::from("/cdn/").join(filename),
        PathBuf::from("data/output").join(filename),
        PathBuf::from("../data/output").join(filename),
        PathBuf::from("../../data/output").join(filename),
    ];

    for path in search_paths {
        debug!(?path, "Checking for file");
        match tokio::fs::read(&path).await {
            Ok(bytes) => {
                debug!(?path, size = bytes.len(), "Successfully read file");
                return Some(bytes);
            }
            Err(e) => {
                debug!(?path, error = ?e, "File not found at path");
            }
        }
    }

    None
}

/// Downloads a file from the CDN via HTTP with retry logic.
///
/// Implements exponential backoff with up to 5 retries to handle transient network issues.
/// Each request has a 30-second timeout to prevent indefinite hangs.
///
/// # Arguments
///   * `filename` - The name of the file to download
///   * `cdn_url` - The base CDN URL
///
/// # Returns
/// The downloaded file contents as bytes
async fn download_file(filename: &str, cdn_url: &str) -> anyhow::Result<Vec<u8>> {
    let url = format!("{cdn_url}/{filename}");
    let max_retries = 5;
    let mut retry_delay = Duration::from_secs(1);
    let mut last_error = None;

    // Timeout to prevent maybe long hangs
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    for attempt in 0..=max_retries {
        debug!(url, attempt, "Downloading file");

        match client.get(&url).send().await {
            Ok(response) => match response.error_for_status() {
                Ok(resp) => match resp.bytes().await {
                    Ok(bytes) => {
                        debug!(url, size = bytes.len(), "Downloaded file");
                        return Ok(bytes.to_vec());
                    }
                    Err(e) => {
                        if attempt < max_retries {
                            warn!(
                                url,
                                attempt,
                                error = ?e,
                                retry_delay_secs = retry_delay.as_secs(),
                                "Failed to read response bytes, retrying"
                            );
                        }
                        last_error = Some(anyhow::Error::from(e));
                        if attempt < max_retries {
                            tokio::time::sleep(retry_delay).await;
                            retry_delay *= 2;
                        }
                    }
                },
                Err(e) => {
                    if attempt < max_retries {
                        warn!(
                            url,
                            attempt,
                            error = ?e,
                            retry_delay_secs = retry_delay.as_secs(),
                            "HTTP error, retrying"
                        );
                    }
                    last_error = Some(anyhow::Error::from(e));
                    if attempt < max_retries {
                        tokio::time::sleep(retry_delay).await;
                        retry_delay *= 2;
                    }
                }
            },
            Err(e) => {
                if attempt < max_retries {
                    warn!(
                        url,
                        attempt,
                        error = ?e,
                        retry_delay_secs = retry_delay.as_secs(),
                        "Request failed, retrying"
                    );
                }
                last_error = Some(anyhow::Error::from(e));
                if attempt < max_retries {
                    tokio::time::sleep(retry_delay).await;
                    retry_delay *= 2;
                }
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| anyhow::anyhow!("Download failed after {} retries", max_retries)))
}

/// Loads a JSON file from disk or downloads it, then parses it.
///
/// # Arguments
///   * `filename` - The name of the JSON file to load
///   * `cdn_url` - The CDN URL to use for downloading if the file is not found locally
///
/// # Returns
/// The parsed JSON value
pub async fn load_json_or_download<T>(filename: &str, cdn_url: &str) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = load_file_or_download(filename, cdn_url).await?;
    let value = serde_json::from_slice(&bytes)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_try_load_from_disk_nonexistent() {
        let result = try_load_from_disk("nonexistent_file_xyz123.json").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_load_file_or_download_invalid_url() {
        let result = load_file_or_download(
            "nonexistent_file_xyz123.json",
            "http://invalid-domain-that-does-not-exist-12345.com",
        )
        .await;
        assert!(result.is_err());
    }
}
