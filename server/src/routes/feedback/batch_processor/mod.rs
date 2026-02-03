use octocrab::Octocrab;
use tracing::{error, info};

const BATCH_LABEL: &str = "batch-in-progress";

/// Find the current open batch PR (if any)
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

/// Update batch PR metadata (title with edit count, labels)
pub async fn update_batch_pr_metadata(
    pr_number: u64,
    edit_request: &super::proposed_edits::EditRequest,
) -> anyhow::Result<()> {
    let Some(personal_token) = crate::external::github::GitHub::github_token() else {
        anyhow::bail!("Failed to get GitHub token");
    };
    
    let octocrab = Octocrab::builder()
        .personal_token(personal_token)
        .build()?;
    
    // Update labels based on edit type
    let mut labels = vec![BATCH_LABEL.to_string(), "webform".to_string()];
    
    // Check edit types using the extract_labels method
    let edit_labels = edit_request.extract_labels_for_batch();
    for label in edit_labels {
        if label != "webform" && !labels.contains(&label) {
            labels.push(label);
        }
    }
    
    match octocrab
        .issues("TUM-Dev", "NavigaTUM")
        .update(pr_number)
        .labels(&labels)
        .send()
        .await
    {
        Ok(_) => info!("Updated labels for batch PR #{}", pr_number),
        Err(e) => error!("Failed to update labels for batch PR #{}: {:?}", pr_number, e),
    }
    
    // Get commits to count edits
    let commits = octocrab
        .pulls("TUM-Dev", "NavigaTUM")
        .pr_commits(pr_number)
        .per_page(100)
        .send()
        .await?;
    
    let edit_count = commits.items.len();
    
    // Update PR title with edit count
    match octocrab
        .issues("TUM-Dev", "NavigaTUM")
        .update(pr_number)
        .title(&format!(
            "chore(data): batch coordinate edits ({} edits)",
            edit_count
        ))
        .send()
        .await
    {
        Ok(_) => info!("Updated title for batch PR #{}", pr_number),
        Err(e) => error!("Failed to update title for batch PR #{}: {:?}", pr_number, e),
    }
    
    Ok(())
}
