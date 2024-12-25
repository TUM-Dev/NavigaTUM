use std::collections::HashMap;
use std::path::Path;

use actix_web::web::{Data, Json};
use actix_web::{post, HttpResponse};
use serde::Deserialize;
use tracing::error;

use crate::limited::hash_map::LimitedHashMap;

use super::github;
use super::proposed_edits::coordinate::Coordinate;
use super::proposed_edits::image::Image;
use super::proposed_edits::tmp_repo::TempRepo;
use super::tokens::RecordedTokens;

mod coordinate;
mod discription;
mod image;
mod tmp_repo;

#[derive(Debug, Deserialize, Clone)]
struct Edit {
    coordinate: Option<Coordinate>,
    image: Option<Image>,
}
pub trait AppliableEdit {
    fn apply(&self, key: &str, base_dir: &Path) -> String;
}

#[derive(Debug, Deserialize)]
pub struct EditRequest {
    token: String,
    edits: LimitedHashMap<String, Edit>,
    additional_context: String,
    privacy_checked: bool,
}

const GIT_URL: &str = "git@github.com:TUM-Dev/NavigaTUM.git";
impl EditRequest {
    #[tracing::instrument]
    async fn apply_changes_and_generate_description(
        &self,
        branch_name: &str,
    ) -> anyhow::Result<String> {
        let repo = TempRepo::clone_and_checkout(GIT_URL, branch_name).await?;
        let desc = repo.apply_and_gen_description(self);
        repo.commit(&desc.title).await?;
        repo.push().await?;
        Ok(desc.body)
    }
    fn edits_for<T: AppliableEdit>(&self, extractor: fn(Edit) -> Option<T>) -> HashMap<String, T> {
        self.edits
            .0
            .clone()
            .into_iter()
            .filter_map(|(k, edit)| extractor(edit).map(|coord| (k, coord)))
            .collect()
    }

    fn extract_labels(&self) -> Vec<String> {
        let mut labels = vec!["webform".to_string()];

        if self
            .edits
            .0
            .iter()
            .any(|(_, edit)| edit.coordinate.is_none())
        {
            labels.push("coordinate".to_string());
        }
        if self.edits.0.iter().any(|(_, edit)| edit.image.is_none()) {
            labels.push("image".to_string());
        }
        labels
    }
    fn extract_subject(&self) -> String {
        let coordinate_edits = self.edits_for(|edit| edit.coordinate);
        let image_edits = self.edits_for(|edit| edit.image);
        match (coordinate_edits.len(), image_edits.len()) {
            (0, 0) => "No Edits".to_string(),
            (1..=5, 0) => format!("Coordinate Edit for {:?}", coordinate_edits.keys()),
            (0, 1..=5) => format!("Added Images for {:?}", image_edits.keys()),
            (0, is) => format!("Added {is} Images"),
            (1..=3, 1..=3) => format!(
                "Edited Images for {:?} and Coordinates for {:?}",
                image_edits.keys(),
                coordinate_edits.keys()
            ),
            (cs, 0) => format!("Edited {cs} Coordinates"),
            (cs, is) => format!("Edited {is} Images and {cs} Coordinates"),
        }
    }
}

#[post("/propose_edit")]
pub async fn propose_edits(
    recorded_tokens: Data<RecordedTokens>,
    req_data: Json<EditRequest>,
) -> HttpResponse {
    // auth
    if let Some(e) = recorded_tokens.validate(&req_data.token).await {
        return e;
    }

    // validate request
    if !req_data.privacy_checked {
        return HttpResponse::UnavailableForLegalReasons()
            .content_type("text/plain")
            .body("Using this endpoint without accepting the privacy policy is not allowed");
    };
    if req_data.edits.0.is_empty() {
        return HttpResponse::UnprocessableEntity()
            .content_type("text/plain")
            .body("Not enough edits provided");
    };
    if req_data.edits.0.len() > 500 {
        return HttpResponse::InsufficientStorage()
            .content_type("text/plain")
            .body("Too many edits provided");
    };

    let branch_name = format!("usergenerated/request-{}", rand::random::<u16>());
    match req_data
        .apply_changes_and_generate_description(&branch_name)
        .await
    {
        Ok(description) => {
            github::open_pr(
                branch_name,
                &format!(
                    "[User-Provided] {subject}",
                    subject = req_data.extract_subject()
                ),
                &description,
                req_data.extract_labels(),
            )
            .await
        }
        Err(e) => {
            error!("Error while applying changes: {e}", e = e);
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("could not apply changes, please try again later")
        }
    }
}
