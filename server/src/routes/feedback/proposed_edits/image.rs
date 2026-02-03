use std::cmp::max;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs::File;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::Deserialize;
use serde::Serialize;
use tracing::error;

use super::AppliableEdit;

/// Sanitizes a key to prevent path traversal attacks.
///
/// Rejects keys that:
/// - Are empty
/// - Contain `..` (path traversal)
/// - Contain `/` or `\` (path separators)
/// - Start with `.` (hidden files)
fn sanitize_key(key: &str) -> anyhow::Result<&str> {
    if key.is_empty() {
        anyhow::bail!("Invalid key: key cannot be empty");
    }
    if key.contains("..") {
        anyhow::bail!("Invalid key: contains path traversal sequence (..)");
    }
    if key.contains('/') {
        anyhow::bail!("Invalid key: contains path separator (/)");
    }
    if key.contains('\\') {
        anyhow::bail!("Invalid key: contains path separator (\\)");
    }
    // Additional safety check: ensure the key doesn't start with a dot
    if key.starts_with('.') {
        anyhow::bail!("Invalid key: cannot start with a dot");
    }
    Ok(key)
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, utoipa::ToSchema)]
pub struct ImageMetadata {
    /// Who created the image
    author: String,
    /// The license under which the image is distributed
    license: Property,
    /// Advanced metadata to control how the image is displayed
    offsets: Option<Offsets>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, utoipa::ToSchema)]
struct Property {
    text: String,
    url: Option<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, utoipa::ToSchema)]
pub struct Offsets {
    header: Option<i32>,
    thumb: Option<i32>,
}
#[derive(Deserialize, Clone, Eq, PartialEq, utoipa::ToSchema)]
pub struct Image {
    /// The image encoded as base64
    #[schema(content_encoding = "base64")]
    content: String,
    metadata: ImageMetadata,
}

impl Image {
    fn apply_metadata_to(
        &self,
        key: &str,
        image_sources: &mut BTreeMap<String, BTreeMap<u32, ImageMetadata>>,
    ) {
        let metadata = self.metadata.clone();
        match image_sources.get_mut(key) {
            Some(t) => {
                let new_key = t.keys().max().unwrap() + 1;
                t.insert(new_key, metadata);
            }
            None => {
                image_sources.insert(key.to_string(), BTreeMap::from([(0, metadata)]));
            }
        }
    }
    fn save_metadata(&self, key: &str, image_dir: &Path) -> anyhow::Result<()> {
        // Sanitize the key to prevent path traversal attacks
        let safe_key = sanitize_key(key)?;

        let file = File::open(image_dir.join("img-sources.yaml"))?;
        let mut image_sources =
            serde_yaml::from_reader::<_, BTreeMap<String, BTreeMap<u32, ImageMetadata>>>(file)?;
        // add the desired change
        self.apply_metadata_to(safe_key, &mut image_sources);
        // save to disk
        let file = File::create(image_dir.join("img-sources.yaml"))?;
        serde_yaml::to_writer(file, &image_sources)?;
        Ok(())
    }
    fn image_should_be_saved_at(key: &str, image_dir: &Path) -> anyhow::Result<PathBuf> {
        // Sanitize the key to prevent path traversal attacks
        let safe_key = sanitize_key(key)?;

        let search_prefix = format!("{safe_key}_");
        let next_free_slot = std::fs::read_dir(image_dir)
            .map_err(|e| anyhow::anyhow!("Failed to read image directory: {}", e))?
            .filter_map(Result::ok)
            .filter_map(|e| e.file_name().to_str().map(String::from))
            .filter(|filename| filename.starts_with(&search_prefix))
            .count();
        Ok(image_dir.join(format!("{safe_key}_{next_free_slot}.webp")))
    }
    fn save_content(&self, target: &Path) -> anyhow::Result<()> {
        let bytes = BASE64_STANDARD.decode(&self.content)?;
        let image = image::load_from_memory(&bytes)?;

        // we scale down too large images to a max of 4k
        if image.width() > 3840 || image.height() > 3840 {
            let crop_factor = 3840.0 / max(image.width(), image.height()) as f32;
            let new_width = crop_factor * image.width() as f32;
            let new_height = crop_factor * image.height() as f32;
            image::imageops::resize(
                &image,
                new_width as u32,
                new_height as u32,
                image::imageops::FilterType::Lanczos3,
            );
        }
        image.save_with_format(target, image::ImageFormat::WebP)?;
        Ok(())
    }
}
impl AppliableEdit for Image {
    fn apply(&self, key: &str, base_dir: &Path, branch: &str) -> String {
        let image_dir = base_dir.join("data").join("sources").join("img");
        let target = match Self::image_should_be_saved_at(key, &image_dir.join("lg")) {
            Ok(path) => path,
            Err(e) => {
                error!(?self, error = ?e, "Invalid key for image");
                return format!("Error: Invalid key - {e}");
            }
        };

        let content_result = self.save_content(&target);
        let metadata_result = self.save_metadata(key, &image_dir);

        let success = format!(
            "![image showing {key}](https://raw.githubusercontent.com/TUM-Dev/NavigaTUM/refs/heads/{branch}/data/sources/img/lg/{filename})",
            filename = target.file_name().unwrap_or_default().to_string_lossy()
        );
        match (content_result, metadata_result) {
            (Ok(()), Ok(())) => success,
            (Err(e), Ok(())) => {
                error!(?self, error = ?e, "Error saving image");
                "Error saving image".to_string()
            }
            (Ok(()), Err(e)) => {
                error!(?self, error = ?e, "Error saving metadata");
                "Error saving metadata".to_string()
            }
            (Err(content), Err(meta)) => {
                error!(?meta, ?content, ?self, "Error saving metadata and content");
                "Error saving metadata+content".to_string()
            }
        }
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("content", &format!("{}KB", self.content.len() / 1024))
            .field("metadata", &self.metadata)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    fn test_image() -> Image {
        Image {
            content: "iVBORw0KGgoAAAANSUhEUgAAAAgAAAAIAQMAAAD+wSzIAAAABlBMVEX///+/v7+jQ3Y5AAAADklEQVQI12P4AIX8EAgALgAD/aNpbtEAAAAASUVORK5CYII=".to_string(),
            metadata: ImageMetadata {
                author: "String".to_string(),
                license: Property {
                    text: "String".to_string(),
                    url: None,
                },
                offsets: None,
            },
        }
    }

    #[test]
    fn test_image_save() {
        let image = test_image();
        let target_dir = tempfile::tempdir().unwrap();
        let target = target_dir.path().join("test.webp");
        image.save_content(&target).unwrap();
        assert!(target.exists());
        let meta = fs::metadata(&target).unwrap();
        assert_eq!(meta.len(), 82);
        assert!(meta.file_type().is_file());
    }

    #[test]
    fn test_metadata_save() {
        let image = test_image();
        let mut image_sources = BTreeMap::new();
        image.apply_metadata_to("01", &mut image_sources);
        assert_eq!(
            image_sources,
            BTreeMap::from([(
                "01".to_string(),
                BTreeMap::from([(0, image.metadata.clone())])
            )])
        );
        image.apply_metadata_to("01", &mut image_sources);
        assert_eq!(
            image_sources,
            BTreeMap::from([(
                "01".to_string(),
                BTreeMap::from([(0, image.metadata.clone()), (1, image.metadata.clone())])
            )])
        );
    }

    #[rstest]
    #[case("mi")]
    #[case("room_01")]
    #[case("building-123")]
    #[case("test_key_123")]
    fn test_sanitize_key_valid(#[case] key: &str) {
        assert!(sanitize_key(key).is_ok());
    }

    #[rstest]
    #[case("")]
    #[case("..")]
    #[case("../")]
    #[case("../../")]
    #[case("../../../cdn/lg/mi")]
    #[case("test/../other")]
    #[case("test/..")]
    #[case("/")]
    #[case("\\")]
    #[case("test/path")]
    #[case("test\\path")]
    #[case("/absolute/path")]
    #[case("C:\\windows\\path")]
    #[case(".hidden")]
    #[case(".")]
    #[case(".secret")]
    #[case("..secret")]
    fn test_sanitize_key_invalid(#[case] key: &str) {
        assert!(sanitize_key(key).is_err());
    }

    #[test]
    fn test_image_should_be_saved_at_with_valid_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = Image::image_should_be_saved_at("test_room", temp_dir.path());
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.file_name().unwrap(), "test_room_0.webp");
    }

    #[test]
    fn test_image_should_be_saved_at_with_traversal_attempt() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = Image::image_should_be_saved_at("../../cdn/lg/mi", temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path traversal"));
    }

    #[test]
    fn test_image_should_be_saved_at_with_absolute_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = Image::image_should_be_saved_at("/etc/passwd", temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path separator"));
    }

    #[test]
    fn test_image_path_stays_within_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = Image::image_should_be_saved_at("valid_key", temp_dir.path());
        assert!(result.is_ok());
        let path = result.unwrap();

        assert!(path.starts_with(temp_dir.path()));
        assert!(!path.to_str().unwrap().contains(".."));
    }
}
