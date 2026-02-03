use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info};

use crate::external::github::GitHub;
use crate::routes::feedback::proposed_edits::{EditRequest, AppliableEdit};
use crate::limited::hash_map::LimitedHashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingEdit {
    pub id: i32,
    pub edit_data: serde_json::Value,
    pub token_id: String,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub window_hours: i64,
    pub max_edits: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            window_hours: std::env::var("BATCH_WINDOW_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(6),
            max_edits: std::env::var("BATCH_MAX_EDITS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
        }
    }
}

impl BatchConfig {
    pub fn is_batch_enabled() -> bool {
        std::env::var("BATCH_ENABLED")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(true)
    }
}

/// Fetch pending edits from the database
#[tracing::instrument(skip(pool))]
pub async fn fetch_pending_edits(pool: &PgPool) -> anyhow::Result<Vec<PendingEdit>> {
    let edits = sqlx::query_as!(
        PendingEdit,
        r#"
        SELECT id, edit_data, token_id, submitted_at
        FROM pending_edit_batches
        WHERE status = 'pending'
        ORDER BY submitted_at ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    
    Ok(edits)
}

/// Group edits into batches based on time window and max size
#[tracing::instrument]
pub fn group_edits_into_batches(
    edits: Vec<PendingEdit>,
    config: &BatchConfig,
) -> Vec<Vec<PendingEdit>> {
    if edits.is_empty() {
        return vec![];
    }
    
    let mut batches = Vec::new();
    let mut current_batch = Vec::new();
    let mut batch_start_time = edits[0].submitted_at;
    
    for edit in edits {
        let time_diff = edit.submitted_at - batch_start_time;
        let hours_diff = time_diff.num_hours();
        
        // Start a new batch if:
        // 1. Time window exceeded
        // 2. Max edits reached
        if hours_diff >= config.window_hours || current_batch.len() >= config.max_edits {
            if !current_batch.is_empty() {
                batches.push(current_batch);
                current_batch = Vec::new();
            }
            batch_start_time = edit.submitted_at;
        }
        
        current_batch.push(edit);
    }
    
    // Add the last batch if it's not empty
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }
    
    batches
}

/// Aggregate multiple EditRequest objects into one
#[tracing::instrument(skip(edits))]
fn aggregate_edit_requests(edits: &[PendingEdit]) -> anyhow::Result<EditRequest> {
    if edits.is_empty() {
        anyhow::bail!("Cannot aggregate empty edit list");
    }
    
    let mut aggregated_edits = HashMap::new();
    let mut additional_contexts = Vec::new();
    
    // Use the first token as the representative token
    let token = edits[0].token_id.clone();
    
    for edit in edits {
        let edit_req: EditRequest = serde_json::from_value(edit.edit_data.clone())?;
        
        // Merge edits
        for (key, value) in edit_req.edits.0.into_iter() {
            aggregated_edits.insert(key, value);
        }
        
        // Collect additional contexts
        if !edit_req.additional_context.is_empty() {
            additional_contexts.push(format!(
                "Edit #{}: {}",
                edit.id,
                edit_req.additional_context
            ));
        }
    }
    
    let aggregated_context = if additional_contexts.is_empty() {
        "Batched coordinate edits".to_string()
    } else {
        additional_contexts.join("\n")
    };
    
    Ok(EditRequest {
        token,
        edits: LimitedHashMap(aggregated_edits),
        additional_context: aggregated_context,
        privacy_checked: true,
    })
}

/// Generate PR description for a batch
#[tracing::instrument(skip(edits))]
fn generate_batch_description(edits: &[PendingEdit]) -> String {
    let count = edits.len();
    let start_time = edits.first().map(|e| e.submitted_at.to_rfc3339()).unwrap_or_default();
    let end_time = edits.last().map(|e| e.submitted_at.to_rfc3339()).unwrap_or_default();
    
    let mut description = format!(
        "## Batched Edit Submission\n\n\
         This PR contains {} coordinate edits submitted between {} and {}.\n\n\
         ### Edits included:\n",
        count, start_time, end_time
    );
    
    for edit in edits {
        if let Ok(edit_req) = serde_json::from_value::<EditRequest>(edit.edit_data.clone()) {
            let subject = edit_req.extract_subject();
            description.push_str(&format!(
                "- Edit #{}: {} ({})\n",
                edit.id,
                subject,
                edit.submitted_at.format("%Y-%m-%d %H:%M:%S")
            ));
        }
    }
    
    description.push_str("\n### Additional context:\n");
    
    for edit in edits {
        if let Ok(edit_req) = serde_json::from_value::<EditRequest>(edit.edit_data.clone()) {
            if !edit_req.additional_context.is_empty() {
                description.push_str(&format!(
                    "**Edit #{}**: {}\n",
                    edit.id,
                    edit_req.additional_context
                ));
            }
        }
    }
    
    description
}

/// Extract labels from a batch of edits
#[tracing::instrument(skip(edits))]
fn extract_batch_labels(edits: &[PendingEdit]) -> Vec<String> {
    let mut labels = vec!["webform".to_string(), "batch".to_string()];
    let mut has_coordinate = false;
    let mut has_image = false;
    
    for edit in edits {
        if let Ok(edit_req) = serde_json::from_value::<EditRequest>(edit.edit_data.clone()) {
            if edit_req.edits.0.iter().any(|(_, e)| e.coordinate.is_some()) {
                has_coordinate = true;
            }
            if edit_req.edits.0.iter().any(|(_, e)| e.image.is_some()) {
                has_image = true;
            }
        }
    }
    
    if has_coordinate {
        labels.push("coordinate".to_string());
    }
    if has_image {
        labels.push("image".to_string());
    }
    
    labels
}

/// Process a batch of edits and create a PR
#[tracing::instrument(skip(pool, edits))]
pub async fn process_batch(pool: &PgPool, edits: Vec<PendingEdit>) -> anyhow::Result<String> {
    if edits.is_empty() {
        anyhow::bail!("Cannot process empty batch");
    }
    
    info!("Processing batch of {} edits", edits.len());
    
    // Mark edits as processing
    let edit_ids: Vec<i32> = edits.iter().map(|e| e.id).collect();
    sqlx::query!(
        "UPDATE pending_edit_batches SET status = 'processing' WHERE id = ANY($1)",
        &edit_ids
    )
    .execute(pool)
    .await?;
    
    // Aggregate edit requests
    let aggregated_request = aggregate_edit_requests(&edits)?;
    
    // Generate branch name
    let branch_name = format!("usergenerated/batch-{}", Utc::now().format("%Y%m%d-%H%M%S"));
    
    // Apply changes and generate description
    match aggregated_request
        .apply_changes_and_generate_description(&branch_name)
        .await
    {
        Ok(auto_description) => {
            // Generate PR title
            let title = format!("chore(data): batch coordinate edits ({} edits)", edits.len());
            
            // Generate PR description
            let batch_description = generate_batch_description(&edits);
            let full_description = format!("{}\n\n---\n\n{}", batch_description, auto_description);
            
            // Extract labels
            let labels = extract_batch_labels(&edits);
            
            // Create PR
            let response = GitHub::default()
                .open_pr(branch_name, &title, &full_description, labels)
                .await;
            
            if response.status().is_success() {
                let pr_url = response.body().to_owned();
                let pr_url_str = String::from_utf8(pr_url.to_vec())
                    .unwrap_or_else(|_| "unknown".to_string());
                
                // Mark edits as completed
                sqlx::query!(
                    "UPDATE pending_edit_batches SET status = 'completed', processed_at = NOW(), batch_pr_url = $1 WHERE id = ANY($2)",
                    &pr_url_str,
                    &edit_ids
                )
                .execute(pool)
                .await?;
                
                info!("Successfully created PR: {}", pr_url_str);
                Ok(pr_url_str)
            } else {
                // Mark edits as failed
                sqlx::query!(
                    "UPDATE pending_edit_batches SET status = 'failed', processed_at = NOW() WHERE id = ANY($1)",
                    &edit_ids
                )
                .execute(pool)
                .await?;
                
                anyhow::bail!("Failed to create PR: {:?}", response.status());
            }
        }
        Err(err) => {
            error!("Failed to apply changes: {:?}", err);
            
            // Mark edits as failed
            sqlx::query!(
                "UPDATE pending_edit_batches SET status = 'failed', processed_at = NOW() WHERE id = ANY($1)",
                &edit_ids
            )
            .execute(pool)
            .await?;
            
            anyhow::bail!("Failed to apply changes: {:?}", err);
        }
    }
}

/// Main entry point for batch processing
#[tracing::instrument(skip(pool))]
pub async fn process_all_batches(pool: &PgPool) -> anyhow::Result<Vec<String>> {
    let config = BatchConfig::default();
    
    info!("Fetching pending edits");
    let pending_edits = fetch_pending_edits(pool).await?;
    
    if pending_edits.is_empty() {
        info!("No pending edits to process");
        return Ok(vec![]);
    }
    
    info!("Found {} pending edits", pending_edits.len());
    
    let batches = group_edits_into_batches(pending_edits, &config);
    info!("Grouped into {} batches", batches.len());
    
    let mut pr_urls = Vec::new();
    
    for (i, batch) in batches.into_iter().enumerate() {
        info!("Processing batch {} with {} edits", i + 1, batch.len());
        match process_batch(pool, batch).await {
            Ok(pr_url) => pr_urls.push(pr_url),
            Err(e) => error!("Failed to process batch {}: {:?}", i + 1, e),
        }
    }
    
    Ok(pr_urls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_group_edits_respects_time_window() {
        let config = BatchConfig {
            window_hours: 6,
            max_edits: 50,
        };

        let base_time = Utc::now();
        let edits = vec![
            PendingEdit {
                id: 1,
                edit_data: serde_json::json!({}),
                token_id: "token1".to_string(),
                submitted_at: base_time,
            },
            PendingEdit {
                id: 2,
                edit_data: serde_json::json!({}),
                token_id: "token2".to_string(),
                submitted_at: base_time + Duration::hours(3),
            },
            PendingEdit {
                id: 3,
                edit_data: serde_json::json!({}),
                token_id: "token3".to_string(),
                submitted_at: base_time + Duration::hours(7), // Should start new batch
            },
        ];

        let batches = group_edits_into_batches(edits, &config);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[1].len(), 1);
    }

    #[test]
    fn test_group_edits_respects_max_size() {
        let config = BatchConfig {
            window_hours: 24,
            max_edits: 2,
        };

        let base_time = Utc::now();
        let edits = vec![
            PendingEdit {
                id: 1,
                edit_data: serde_json::json!({}),
                token_id: "token1".to_string(),
                submitted_at: base_time,
            },
            PendingEdit {
                id: 2,
                edit_data: serde_json::json!({}),
                token_id: "token2".to_string(),
                submitted_at: base_time + Duration::hours(1),
            },
            PendingEdit {
                id: 3,
                edit_data: serde_json::json!({}),
                token_id: "token3".to_string(),
                submitted_at: base_time + Duration::hours(2), // Should start new batch
            },
        ];

        let batches = group_edits_into_batches(edits, &config);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[1].len(), 1);
    }

    #[test]
    fn test_empty_edits_returns_empty_batches() {
        let config = BatchConfig::default();
        let batches = group_edits_into_batches(vec![], &config);
        assert_eq!(batches.len(), 0);
    }
}
