use std::collections::BTreeMap;
use std::fmt::{self, Debug};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::prelude::BASE64_STANDARD;
use image::imageops::FilterType;
use image::{ImageFormat, load_from_memory};
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
                let new_key = t
                    .keys()
                    .max()
                    .expect("entry in image_sources always has at least one image")
                    + 1;
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
        Self::update_metadata_file(image_dir, |image_sources| {
            self.apply_metadata_to(safe_key, image_sources);
        })
    }
    fn update_metadata_file(
        image_dir: &Path,
        update: impl FnOnce(&mut BTreeMap<String, BTreeMap<u32, ImageMetadata>>),
    ) -> anyhow::Result<()> {
        let file = File::open(image_dir.join("img-sources.yaml"))?;
        let mut image_sources =
            serde_yaml::from_reader::<_, BTreeMap<String, BTreeMap<u32, ImageMetadata>>>(file)?;
        update(&mut image_sources);
        let file = File::create(image_dir.join("img-sources.yaml"))?;
        serde_yaml::to_writer(file, &image_sources)?;
        Ok(())
    }
    /// Replaces the key's images instead of appending a slot: slot 0 is overwritten, stray
    /// higher slots are removed, and the key's img-sources entry is reset to the new metadata.
    pub(super) fn replace(
        &self,
        key: &str,
        base_dir: &Path,
        branch: &str,
    ) -> anyhow::Result<String> {
        let safe_key = sanitize_key(key)?;
        let image_dir = base_dir.join("data").join("sources").join("img");
        let lg_dir = image_dir.join("lg");
        let filename = format!("{safe_key}_0.webp");
        let slot_prefix = format!("{safe_key}_");
        for entry in fs::read_dir(&lg_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str()
                && name.starts_with(&slot_prefix)
                && name != filename
            {
                fs::remove_file(entry.path())?;
            }
        }
        self.save_content(&lg_dir.join(&filename))
            .inspect_err(|e| error!(?self, error = ?e, "Error replacing image"))?;
        Self::update_metadata_file(&image_dir, |image_sources| {
            image_sources.insert(
                safe_key.to_string(),
                BTreeMap::from([(0, self.metadata.clone())]),
            );
        })?;
        Ok(Self::summary_markdown(key, branch, &filename))
    }
    fn summary_markdown(key: &str, branch: &str, filename: &str) -> String {
        format!(
            "![image showing {key}]({})",
            Self::raw_url(branch, filename)
        )
    }
    fn raw_url(branch: &str, filename: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/TUM-Dev/NavigaTUM/refs/heads/{branch}/data/sources/img/lg/{filename}"
        )
    }
    /// Raw URL of the canonical (slot-0) image for `key` on `branch`.
    pub(super) fn raw_lg_url(key: &str, branch: &str) -> anyhow::Result<String> {
        let safe_key = sanitize_key(key)?;
        Ok(Self::raw_url(branch, &format!("{safe_key}_0.webp")))
    }
    fn image_should_be_saved_at(key: &str, image_dir: &Path) -> anyhow::Result<PathBuf> {
        // Sanitize the key to prevent path traversal attacks
        let safe_key = sanitize_key(key)?;

        let search_prefix = format!("{safe_key}_");
        let next_free_slot = fs::read_dir(image_dir)
            .map_err(|e| anyhow::anyhow!("Failed to read image directory: {e}"))?
            .filter_map(Result::ok)
            .filter_map(|e| e.file_name().to_str().map(String::from))
            .filter(|filename| filename.starts_with(&search_prefix))
            .count();
        Ok(image_dir.join(format!("{safe_key}_{next_free_slot}.webp")))
    }
    /// Author the submitter named for this image, as it was committed to img-sources.yaml.
    pub(super) fn author(&self) -> &str {
        &self.metadata.author
    }
    /// Pixel dimensions of the upload, before any resize.
    pub(super) fn decoded_dimensions(&self) -> anyhow::Result<(u32, u32)> {
        let bytes = BASE64_STANDARD.decode(&self.content)?;
        let image = load_from_memory(&bytes)?;
        Ok((image.width(), image.height()))
    }
    fn save_content(&self, target: &Path) -> anyhow::Result<()> {
        let bytes = BASE64_STANDARD.decode(&self.content)?;
        let mut image = load_from_memory(&bytes)?;

        // scale oversized uploads down to a 4k longest side, fitting within while preserving aspect ratio.
        if image.width() > 3840 || image.height() > 3840 {
            image = image.resize(3840, 3840, FilterType::Lanczos3);
        }
        image.save_with_format(target, ImageFormat::WebP)?;
        Ok(())
    }
}
impl AppliableEdit for Image {
    fn apply(&self, key: &str, base_dir: &Path, branch: &str) -> anyhow::Result<String> {
        let image_dir = base_dir.join("data").join("sources").join("img");
        let target = Self::image_should_be_saved_at(key, &image_dir.join("lg"))
            .inspect_err(|e| error!(?self, error = ?e, "Invalid key for image"))?;

        self.save_content(&target)
            .inspect_err(|e| error!(?self, error = ?e, "Error saving image"))?;
        self.save_metadata(key, &image_dir)
            .inspect_err(|e| error!(?self, error = ?e, "Error saving metadata"))?;

        Ok(Self::summary_markdown(
            key,
            branch,
            &target.file_name().unwrap_or_default().to_string_lossy(),
        ))
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("content", &format!("{}KB", self.content.len() / 1024))
            .field("metadata", &self.metadata)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        reason = "tests assert via panic/unwrap"
    )]
    use std::fs;
    use std::io::Cursor;

    use image::{DynamicImage, RgbImage};
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

    /// A solid-color image of the given size, base64-encoded as PNG like a real upload.
    fn image_of_size(width: u32, height: u32) -> Image {
        let mut bytes = Vec::new();
        DynamicImage::ImageRgb8(RgbImage::new(width, height))
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        Image {
            content: BASE64_STANDARD.encode(&bytes),
            ..test_image()
        }
    }

    #[rstest]
    // longest side >4k: downscaled to a 3840 longest side, aspect ratio preserved.
    #[case(5000, 2000, 3840, 1536)]
    #[case(2000, 5000, 1536, 3840)]
    // both sides ≤4k: committed unchanged.
    #[case(1920, 1080, 1920, 1080)]
    #[case(3840, 3840, 3840, 3840)]
    fn test_image_save_caps_dimensions(
        #[case] in_width: u32,
        #[case] in_height: u32,
        #[case] out_width: u32,
        #[case] out_height: u32,
    ) {
        let target_dir = tempfile::tempdir().unwrap();
        let target = target_dir.path().join("test.webp");
        image_of_size(in_width, in_height)
            .save_content(&target)
            .unwrap();
        let saved = load_from_memory(&fs::read(&target).unwrap()).unwrap();
        assert_eq!((saved.width(), saved.height()), (out_width, out_height));
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
