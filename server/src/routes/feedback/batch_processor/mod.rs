use tracing::{error, info};

use crate::external::github::GitHub;

const BATCH_LABEL: &str = "batch-in-progress";

/// Find the current open batch PR (if any)
pub async fn find_open_batch_pr() -> anyhow::Result<Option<(u64, String)>> {
    let github = GitHub::default();

    match github.find_pr_with_label(BATCH_LABEL).await {
        Ok(Some((pr_number, branch))) => {
            info!("Found open batch PR: #{} ({})", pr_number, branch);
            Ok(Some((pr_number, branch)))
        }
        Ok(None) => {
            info!("No open batch PR found");
            Ok(None)
        }
        Err(e) => {
            error!("Error finding batch PR: {:?}", e);
            Err(e)
        }
    }
}

/// Update batch PR metadata (title with edit count, labels, and description)
pub async fn update_batch_pr_metadata(
    pr_number: u64,
    edit_request: &super::proposed_edits::EditRequest,
    new_edit_description: &str,
) -> anyhow::Result<()> {
    // Update labels based on edit type
    let mut labels = vec![BATCH_LABEL.to_string(), "webform".to_string()];

    // Check edit types using the extract_labels method
    let edit_labels = edit_request.extract_labels();
    for label in edit_labels {
        if label != "webform" && !labels.contains(&label) {
            labels.push(label);
        }
    }

    let github = GitHub::default();
    match github.update_pr_labels(pr_number, labels).await {
        Ok(_) => info!("Updated labels for batch PR #{}", pr_number),
        Err(e) => error!(
            error=?e, %pr_number, "Failed to update labels for batch PR"
        ),
    }

    // Get commits to count edits
    let github = GitHub::default();
    let edit_count = match github.get_pr_commit_count(pr_number).await {
        Ok(count) => count,
        Err(e) => {
            error!(error=?e, %pr_number, "Failed to get commit count for PR");
            return Err(e);
        }
    };

    // Update PR title with edit count
    let github = GitHub::default();
    let title = format!("chore(data): batch coordinate edits ({edit_count} edits)");
    match github.update_pr_title(pr_number, title).await {
        Ok(_) => info!(%pr_number, "Updated title for batch PR"),
        Err(e) => error!(error=?e, %pr_number, "Failed to update title for batch PR"),
    }

    // Append to PR description
    let github = GitHub::default();
    let current_description = github
        .get_pr_description(pr_number)
        .await
        .unwrap_or_default();

    // Append the new edit's description
    let updated_description = if current_description.is_empty() {
        format!(            "## Batched Coordinate Edits\n\n### Edit #{edit_count}\n{new_edit_description}")
    } else {
        format!("{current_description}\n\n---\n\n### Edit #{edit_count}\n{new_edit_description}")
    };

    let github = GitHub::default();
    match github
        .update_pr_description(pr_number, updated_description)
        .await
    {
        Ok(_) => info!("Updated description for batch PR #{}", pr_number),
        Err(e) => error!(
            "Failed to update description for batch PR #{}: {:?}",
            pr_number, e
        Ok(_) => info!(%pr_number, "Updated description for batch PR"),
        Err(e) => error!(error=?e, %pr_number, 
            "Failed to update description for batch PR"),
    }

    Ok(())
}
