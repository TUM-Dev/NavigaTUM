use chrono::Utc;
use octocrab::Octocrab;
use tracing::{error, info};

use crate::routes::feedback::proposed_edits::EditRequest;

const BATCH_LABEL: &str = "batch-in-progress";
const BATCH_BRANCH_PREFIX: &str = "usergenerated/batch-";

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
        if let Some(labels) = &pr.labels {
            for label in labels {
                if label.name == BATCH_LABEL {
                    info!("Found open batch PR: #{} ({})", pr.number, pr.head.ref_field);
                    return Ok(Some((pr.number, pr.head.ref_field)));
                }
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
            "chore(data): batch coordinate edits",
            &branch_name,
            "main",
        )
        .body(&format!(
            "## Batched Edit Submission\n\n\
             This PR collects coordinate edits submitted over time.\n\
             New edits are added as commits to this PR.\n\n\
             Started at: {}\n\n\
             ### Edits included:\n\
             *(Edits will appear below as they are submitted)*\n\n",
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

/// Add an edit to the current batch PR and update its description
#[tracing::instrument(skip(edit_request))]
pub async fn add_edit_to_batch_pr(edit_request: &EditRequest) -> anyhow::Result<String> {
    let (pr_number, branch_name) = get_or_create_batch_pr().await?;
    
    info!("Adding edit to batch PR #{}", pr_number);
    
    // Apply the changes and push to the branch
    match edit_request
        .apply_changes_and_generate_description(&branch_name)
        .await
    {
        Ok(_description) => {
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
            
            // Get commits to count edits
            let commits = octocrab
                .pulls("TUM-Dev", "NavigaTUM")
                .pr_commits(pr_number)
                .per_page(100)
                .send()
                .await?;
            
            let edit_count = commits.items.len();
            
            // Update PR title with edit count
            let _ = octocrab
                .issues("TUM-Dev", "NavigaTUM")
                .update(pr_number)
                .title(&format!("chore(data): batch coordinate edits ({} edits)", edit_count))
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
