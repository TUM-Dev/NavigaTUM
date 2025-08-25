use crate::proposed_edits::AppliableEdit;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use log::error;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::max;
use std::collections::BTreeMap;
use std::error;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Source {
    author: String,
    license: Property,
    source: Property,
    #[serde(skip_serializing_if = "Option::is_none")]
    offsets: Option<Offsets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<BTreeMap<String, String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
struct Property {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Offsets {
    #[serde(skip_serializing_if = "Option::is_none")]
    header: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumb: Option<i32>,
}
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Image {
    content: String,
    metadata: Source,
}

impl Image {
    fn apply_metadata_to(
        &self,
        key: &str,
        image_sources: &mut BTreeMap<String, BTreeMap<u32, Source>>,
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
    fn save_metadata(&self, key: &str, image_dir: &Path) -> Result<(), Box<dyn error::Error>> {
        let file = File::open(image_dir.join("img-sources.yaml"))?;
        let mut image_sources =
            serde_yaml::from_reader::<_, BTreeMap<String, BTreeMap<u32, Source>>>(file)?;
        // add the desired change
        self.apply_metadata_to(key, &mut image_sources);
        // save to disk
        let file = File::create(image_dir.join("img-sources.yaml"))?;
        serde_yaml::to_writer(file, &image_sources)?;
        Ok(())
    }
    fn image_should_be_saved_at(key: &str, image_dir: &Path) -> PathBuf {
        let image_dir = image_dir.join("lg");
        let search_prefix = format!("{key}_");
        let next_free_slot = std::fs::read_dir(image_dir.clone())
            .unwrap()
            .filter_map(|res| res.ok())
            .map(|e| e.file_name().to_str().unwrap().to_string())
            .filter(|filename| filename.starts_with(&search_prefix))
            .count()
            + 1;
        image_dir.join(format!("{key}_{next_free_slot}.webp"))
    }
    fn save_content(&self, target: &Path) -> Result<(), Box<dyn error::Error>> {
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
    fn apply(&self, key: &str, base_dir: &Path) -> String {
        let image_dir = base_dir.join("data").join("images");
        let target = Self::image_should_be_saved_at(key, &image_dir);

        let content_result = self.save_content(&target);
        let metadata_result = self.save_metadata(key, &image_dir);

        let success=format!("<img src='data:image/png;base64,{content}' alt='Full image for {key}' height='50%' />  Layout", content=self.content);
        match (content_result, metadata_result) {
            (Ok(()), Ok(())) => success,
            (Err(e), Ok(())) => {
                error!("Error saving image: {e:?} for {self:?}");
                "Error saving image".to_string()
            }
            (Ok(()), Err(e)) => {
                error!("Error saving metadata: {e:?} for {self:?}");
                "Error saving metadata".to_string()
            }
            (Err(content), Err(meta)) => {
                error!("Error saving metadata: {meta:?} for {self:?}");
                error!("Error saving content: {content:?} for {self:?}");
                "Error saving metadata+content".to_string()
            }
        }
    }
}

#[cfg(test)]
mod image_tests {
    use super::*;
    use std::fs;

    fn test_image() -> Image {
        Image {
            content: "iVBORw0KGgoAAAANSUhEUgAAAAgAAAAIAQMAAAD+wSzIAAAABlBMVEX///+/v7+jQ3Y5AAAADklEQVQI12P4AIX8EAgALgAD/aNpbtEAAAAASUVORK5CYII=".to_string(),
            metadata: Source {
                author: "String".to_string(),
                license: Property {
                    text: "String".to_string(),
                    url: None,
                },
                source: Property {
                    text: "String".to_string(),
                    url: None,
                },
                offsets: None,
                meta: None,
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
}
