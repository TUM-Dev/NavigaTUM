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
            "Failed to update labels for batch PR #{}: {:?}",
            pr_number, e
        ),
    }

    // Get commits to count edits
    let github = GitHub::default();
    let edit_count = match github.get_pr_commit_count(pr_number).await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to get commit count for PR #{}: {:?}", pr_number, e);
            return Err(e);
        }
    };

    // Update PR title with edit count
    let github = GitHub::default();
    let title = format!("chore(data): batch coordinate edits ({} edits)", edit_count);
    match github.update_pr_title(pr_number, title).await {
        Ok(_) => info!("Updated title for batch PR #{}", pr_number),
        Err(e) => error!(
            "Failed to update title for batch PR #{}: {:?}",
            pr_number, e
        ),
    }

    // Append to PR description
    let github = GitHub::default();
    let current_description = github
        .get_pr_description(pr_number)
        .await
        .unwrap_or_default();

    // Append the new edit's description
    let updated_description = if current_description.is_empty() {
        format!(
            "## Batched Coordinate Edits\n\n### Edit #{}\n{}",
            edit_count, new_edit_description
        )
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
        ),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_label_constant() {
        assert_eq!(BATCH_LABEL, "batch-in-progress");
    }

    #[test]
    fn test_description_formatting_first_edit() {
        let description = "Test edit description";
        let edit_count = 1;
        let current_description = String::new();

        let result = if current_description.is_empty() {
            format!(
                "## Batched Coordinate Edits\n\n### Edit #{}\n{}",
                edit_count, description
            )
        } else {
            format!(
                "{}\n\n---\n\n### Edit #{}\n{}",
                current_description, edit_count, description
            )
        };

        assert!(result.starts_with("## Batched Coordinate Edits"));
        assert!(result.contains("### Edit #1"));
        assert!(result.contains("Test edit description"));
    }

    #[test]
    fn test_description_formatting_subsequent_edit() {
        let new_description = "Second edit";
        let edit_count = 2;
        let current_description = "## Batched Coordinate Edits\n\n### Edit #1\nFirst edit";

        let result = format!(
            "{}\n\n---\n\n### Edit #{}\n{}",
            current_description, edit_count, new_description
        );

        assert!(result.contains("### Edit #1"));
        assert!(result.contains("First edit"));
        assert!(result.contains("---"));
        assert!(result.contains("### Edit #2"));
        assert!(result.contains("Second edit"));
    }

    #[test]
    fn test_label_deduplication() {
        let mut labels = vec![BATCH_LABEL.to_string(), "webform".to_string()];
        let edit_labels = vec!["webform".to_string(), "coordinate".to_string()];

        for label in edit_labels {
            if label != "webform" && !labels.contains(&label) {
                labels.push(label);
            }
        }

        // Should have batch-in-progress, webform, and coordinate
        assert_eq!(labels.len(), 3);
        assert!(labels.contains(&"batch-in-progress".to_string()));
        assert!(labels.contains(&"webform".to_string()));
        assert!(labels.contains(&"coordinate".to_string()));
    }
}
