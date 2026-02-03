use chrono::Utc;
use octocrab::Octocrab;
use tracing::{error, info};

use crate::routes::feedback::proposed_edits::EditRequest;

const BATCH_LABEL: &str = "batch-in-progress";
const BATCH_BRANCH_PREFIX: &str = "usergenerated/batch-";

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

/// Find the current open batch PR (if any)
#[tracing::instrument]
pub async fn find_open_batch_pr() -> anyhow::Result<Option<(u64, String)>> {
    let Some(personal_token) = crate::external::github::GitHub::github_token() else {
        anyhow::bail!("Failed to get GitHub token");
    };
    
    let octocrab = Octocrab::builder()
        .personal_token(personal_token)
        .build()?;
    
    // Search for open PRs with the batch-in-progress label
    let pulls = octocrab
        .pulls("TUM-Dev", "NavigaTUM")
        .list()
        .state(octocrab::params::State::Open)
        .per_page(100)
        .send()
        .await?;
    
    // Find a PR with the batch label
    for pr in pulls.items {
        // Check if any label matches BATCH_LABEL
        for label in &pr.labels {
            if label.name == BATCH_LABEL {
                info!("Found open batch PR: #{} ({})", pr.number, pr.head.ref_field);
                return Ok(Some((pr.number, pr.head.ref_field)));
            }
        }
    }
    
    info!("No open batch PR found");
    Ok(None)
}

/// Create a new batch PR or return the existing one
#[tracing::instrument]
pub async fn get_or_create_batch_pr() -> anyhow::Result<(u64, String)> {
    // Check if there's already an open batch PR
    if let Some((pr_number, branch)) = find_open_batch_pr().await? {
        return Ok((pr_number, branch));
    }
    
    // Create a new batch PR
    let branch_name = format!("{}{}", BATCH_BRANCH_PREFIX, Utc::now().format("%Y%m%d-%H%M%S"));
    
    // Create an initial empty commit to start the PR
    let Some(personal_token) = crate::external::github::GitHub::github_token() else {
        anyhow::bail!("Failed to get GitHub token");
    };
    
    let octocrab = Octocrab::builder()
        .personal_token(personal_token)
        .build()?;
    
    // Create the PR with initial description
    let pr = octocrab
        .pulls("TUM-Dev", "NavigaTUM")
        .create(
            "chore(data): batch coordinate edits (in progress)",
            &branch_name,
            "main",
        )
        .body(&format!(
            "## Batched Edit Submission (In Progress)\n\n\
             This PR collects coordinate edits submitted over time.\n\
             New edits will be added as commits to this PR.\n\n\
             Started at: {}\n\n\
             ### Edits included:\n\
             *(Edits will appear as they are submitted)*",
            Utc::now().to_rfc3339()
        ))
        .maintainer_can_modify(true)
        .send()
        .await?;
    
    let pr_number = pr.number;
    
    // Add the batch-in-progress label
    octocrab
        .issues("TUM-Dev", "NavigaTUM")
        .update(pr_number)
        .labels(&[
            BATCH_LABEL.to_string(),
            "webform".to_string(),
        ])
        .assignees(&["CommanderStorm".to_string()])
        .send()
        .await?;
    
    info!("Created new batch PR: #{} ({})", pr_number, branch_name);
    Ok((pr_number, branch_name))
}

/// Add an edit to the current batch PR
#[tracing::instrument(skip(edit_request))]
pub async fn add_edit_to_batch_pr(edit_request: &EditRequest) -> anyhow::Result<String> {
    let (pr_number, branch_name) = get_or_create_batch_pr().await?;
    
    info!("Adding edit to batch PR #{}", pr_number);
    
    // Apply the changes and push to the branch
    match edit_request
        .apply_changes_and_generate_description(&branch_name)
        .await
    {
        Ok(description) => {
            info!("Successfully added edit to batch PR #{}", pr_number);
            
            // Update PR labels based on the edit type
            let mut labels = vec![BATCH_LABEL.to_string(), "webform".to_string()];
            if edit_request.edits.0.iter().any(|(_, e)| e.coordinate.is_some()) {
                labels.push("coordinate".to_string());
            }
            if edit_request.edits.0.iter().any(|(_, e)| e.image.is_some()) {
                labels.push("image".to_string());
            }
            
            let Some(personal_token) = crate::external::github::GitHub::github_token() else {
                anyhow::bail!("Failed to get GitHub token");
            };
            
            let octocrab = Octocrab::builder()
                .personal_token(personal_token)
                .build()?;
            
            // Update labels
            let _ = octocrab
                .issues("TUM-Dev", "NavigaTUM")
                .update(pr_number)
                .labels(&labels)
                .send()
                .await;
            
            // Get the PR URL
            let pr = octocrab
                .pulls("TUM-Dev", "NavigaTUM")
                .get(pr_number)
                .await?;
            
            Ok(pr.html_url.map(|u| u.to_string()).unwrap_or_else(|| format!("https://github.com/TUM-Dev/NavigaTUM/pull/{}", pr_number)))
        }
        Err(e) => {
            error!("Failed to apply changes: {:?}", e);
            anyhow::bail!("Failed to apply changes: {:?}", e)
        }
    }
}

/// Finalize the current batch PR (remove the in-progress label)
#[tracing::instrument]
pub async fn finalize_batch_pr() -> anyhow::Result<()> {
    let Some((pr_number, _)) = find_open_batch_pr().await? else {
        info!("No open batch PR to finalize");
        return Ok(());
    };
    
    info!("Finalizing batch PR #{}", pr_number);
    
    let Some(personal_token) = crate::external::github::GitHub::github_token() else {
        anyhow::bail!("Failed to get GitHub token");
    };
    
    let octocrab = Octocrab::builder()
        .personal_token(personal_token)
        .build()?;
    
    // Get current PR to read its labels and edit count
    let pr = octocrab
        .pulls("TUM-Dev", "NavigaTUM")
        .get(pr_number)
        .await?;
    
    // Count commits as a proxy for edit count
    let commits = octocrab
        .pulls("TUM-Dev", "NavigaTUM")
        .pr_commits(pr_number)
        .per_page(100)
        .send()
        .await?;
    
    let edit_count = commits.items.len();
    
    // Remove the batch-in-progress label and update title
    let mut labels: Vec<String> = pr
        .labels
        .iter()
        .filter(|l| l.name != BATCH_LABEL)
        .map(|l| l.name.clone())
        .collect();
    
    // Ensure webform label is present
    if !labels.contains(&"webform".to_string()) {
        labels.push("webform".to_string());
    }
    
    octocrab
        .issues("TUM-Dev", "NavigaTUM")
        .update(pr_number)
        .title(&format!("chore(data): batch coordinate edits ({} edits)", edit_count))
        .labels(&labels)
        .send()
        .await?;
    
    info!("Finalized batch PR #{} with {} edits", pr_number, edit_count);
    Ok(())
}
