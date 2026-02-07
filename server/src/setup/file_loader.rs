use std::path::PathBuf;
use tracing::{debug, info};

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
/// 1. `./data/output/{filename}` - relative to current working directory
/// 2. `../data/output/{filename}` - one level up (useful when running from server/ directory)
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

/// Downloads a file from the CDN via HTTP.
///
/// # Arguments
///   * `filename` - The name of the file to download
///   * `cdn_url` - The base CDN URL
///
/// # Returns
/// The downloaded file contents as bytes
async fn download_file(filename: &str, cdn_url: &str) -> anyhow::Result<Vec<u8>> {
    let url = format!("{cdn_url}/{filename}");
    debug!(url, "Downloading file");

    let response = reqwest::get(&url).await?.error_for_status()?;

    let bytes = response.bytes().await?;
    debug!(url, size = bytes.len(), "Downloaded file");

    Ok(bytes.to_vec())
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
