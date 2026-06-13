use std::env;
use std::sync::LazyLock;

use actix_web::HttpResponse;
use octocrab::Octocrab;
use octocrab::params::State;
use regex::Regex;
use tracing::error;

static FEEDBACK_NEWLINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[ \t]*\n").expect("static regex must compile at startup"));

/// Identifiers for a freshly-opened pull request.
///
/// `node_id` is the GraphQL global ID, required by mutations like
/// `enablePullRequestAutoMerge` that don't accept the REST `number`.
#[derive(Debug)]
pub struct PrCreated {
    pub number: u64,
    pub node_id: String,
    pub html_url: String,
}

#[derive(Debug)]
pub struct GitHub {
    octocrab: Option<Octocrab>,
}
impl Default for GitHub {
    fn default() -> Self {
        let octocrab = if let Some(personal_token) = Self::github_token() {
            Octocrab::builder()
                .personal_token(personal_token)
                .build()
                .map_err(|e| error!(error = ?e, "Could not create Octocrab instance"))
                .ok()
        } else {
            None
        };
        Self { octocrab }
    }
}
impl GitHub {
    #[tracing::instrument(skip(description))]
    pub async fn open_issue(
        self,
        title: &str,
        description: &str,
        labels: Vec<String>,
    ) -> HttpResponse {
        let title = Self::clean_feedback_data(title, 512);
        let description = Self::clean_feedback_data(description, 1024 * 1024);

        if title.len() < 3 || description.len() < 10 {
            return HttpResponse::UnprocessableEntity()
                .content_type("text/plain")
                .body("Subject or body missing or too short");
        }
        let Some(octocrab) = self.octocrab else {
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed to create issue, please try again later");
        };

        // Box::pin keeps octocrab's large send future off the caller's stack frame (clippy::large_futures).
        let resp = Box::pin(
            octocrab
                .issues("TUM-Dev", "navigatum")
                .create(title)
                .body(description)
                .labels(labels)
                .send(),
        )
        .await;

        match resp {
            Ok(issue) => HttpResponse::Created()
                .content_type("text/plain")
                .body(issue.html_url.to_string()),
            Err(e) => {
                error!(error = ?e, "Error creating issue");
                HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("Failed to create issue, please try again later")
            }
        }
    }

    #[tracing::instrument(skip(description))]
    pub async fn open_pr(
        self,
        branch: String,
        title: &str,
        description: &str,
        labels: Vec<String>,
    ) -> Result<PrCreated, HttpResponse> {
        let Some(octocrab) = self.octocrab else {
            return Err(HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed to create a pull request, please try again later"));
        };

        // create the PR
        let (pr_number, node_id) = match octocrab
            .pulls("TUM-Dev", "NavigaTUM")
            .create(title, branch, "main")
            .body(description)
            .maintainer_can_modify(true)
            .send()
            .await
        {
            Ok(pr) => {
                if let (Some(n), Some(id)) = (pr.number, pr.node_id) {
                    (n, id)
                } else {
                    error!("GitHub returned a created PR without a number or node_id");
                    return Err(HttpResponse::InternalServerError()
                        .content_type("text/plain")
                        .body("Failed to create a pull request, please try again later"));
                }
            }
            Err(e) => {
                error!(error = ?e, "Error creating pull request");
                return Err(HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("Failed to create a pull request, please try again later"));
            }
        };

        // For some reason the labels and assignees cannot be set via the create call, but must be updated afterwards
        let resp = octocrab
            .issues("TUM-Dev", "navigatum")
            .update(pr_number)
            .labels(&labels)
            .assignees(&["CommanderStorm".to_string()])
            .send()
            .await;

        match resp {
            Ok(issue) => Ok(PrCreated {
                number: pr_number,
                node_id,
                html_url: issue.html_url.to_string(),
            }),
            Err(e) => {
                error!(error = ?e, "Error updating PR");
                Err(HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("Failed to create a pull request, please try again later"))
            }
        }
    }

    /// Look up a pull request's GraphQL global ID (`node_id`) by REST number.
    ///
    /// Needed to feed `enablePullRequestAutoMerge` / `disablePullRequestAutoMerge`,
    /// which only accept GraphQL IDs.
    #[tracing::instrument]
    pub async fn pr_node_id(self, pr_number: u64) -> anyhow::Result<String> {
        let Some(octocrab) = self.octocrab else {
            anyhow::bail!("GitHub client not initialized");
        };

        let pr = octocrab
            .pulls("TUM-Dev", "NavigaTUM")
            .get(pr_number)
            .await?;

        pr.node_id
            .ok_or_else(|| anyhow::anyhow!("GitHub returned PR #{pr_number} without a node_id"))
    }

    /// Enable squash auto-merge on a PR. Idempotent failures (already enabled) bubble up as `Err`.
    ///
    /// The repo only allows squash merges (`allow_squash_merge: true`, the others false),
    /// so the merge method is hardcoded.
    #[tracing::instrument]
    pub async fn enable_auto_merge_squash(self, pr_node_id: &str) -> anyhow::Result<()> {
        let Some(octocrab) = self.octocrab else {
            anyhow::bail!("GitHub client not initialized");
        };

        let query = "mutation($prId: ID!) { \
            enablePullRequestAutoMerge(input: { pullRequestId: $prId, mergeMethod: SQUASH }) { \
                clientMutationId \
            } \
        }";
        let _: serde_json::Value = octocrab
            .graphql(&serde_json::json!({
                "query": query,
                "variables": { "prId": pr_node_id },
            }))
            .await?;
        Ok(())
    }

    /// Disable auto-merge on a PR. Returns `Err` if it wasn't enabled (caller may ignore).
    #[tracing::instrument]
    pub async fn disable_auto_merge(self, pr_node_id: &str) -> anyhow::Result<()> {
        let Some(octocrab) = self.octocrab else {
            anyhow::bail!("GitHub client not initialized");
        };

        let query = "mutation($prId: ID!) { \
            disablePullRequestAutoMerge(input: { pullRequestId: $prId }) { \
                clientMutationId \
            } \
        }";
        let _: serde_json::Value = octocrab
            .graphql(&serde_json::json!({
                "query": query,
                "variables": { "prId": pr_node_id },
            }))
            .await?;
        Ok(())
    }

    /// Remove all returns a string, which has
    /// - all control characters removed
    /// - is at most len characters long
    /// - can be nicely formatted in markdown (just \n in md is not a linebreak)
    fn clean_feedback_data(s: &str, len: usize) -> String {
        let s_clean = s
            .chars()
            .filter(|c| !c.is_control() || (c == &'\n'))
            .take(len)
            .collect::<String>();

        FEEDBACK_NEWLINE_RE
            .replace_all(&s_clean, "  \n")
            .to_string()
    }

    pub fn github_token() -> Option<String> {
        match env::var("GITHUB_TOKEN") {
            Ok(token) => Some(token.trim().to_string()),
            Err(e) => {
                error!(error = ?e, "GITHUB_TOKEN has to be set for feedback");
                None
            }
        }
    }

    /// Find an open PR with a specific label
    #[tracing::instrument]
    pub async fn find_pr_with_label(self, label: &str) -> anyhow::Result<Option<(u64, String)>> {
        let Some(octocrab) = self.octocrab else {
            anyhow::bail!("GitHub client not initialized");
        };

        // Search through all pages of open PRs to find one with the label
        let mut page = octocrab
            .pulls("TUM-Dev", "NavigaTUM")
            .list()
            .state(State::Open)
            .per_page(100)
            .send()
            .await?;

        loop {
            for pr in &page.items {
                for pr_label in pr.labels.iter().flatten() {
                    if pr_label.name == label {
                        let (Some(head), Some(number)) = (pr.head.as_deref(), pr.number) else {
                            continue;
                        };
                        return Ok(Some((number, head.ref_field.clone())));
                    }
                }
            }

            // Check if there's a next page
            match octocrab.get_page(&page.next).await? {
                Some(next_page) => page = next_page,
                None => break,
            }
        }

        Ok(None)
    }

    /// Update PR labels
    #[tracing::instrument]
    pub async fn update_pr_labels(self, pr_number: u64, labels: Vec<String>) -> anyhow::Result<()> {
        let Some(octocrab) = self.octocrab else {
            anyhow::bail!("GitHub client not initialized");
        };

        octocrab
            .issues("TUM-Dev", "NavigaTUM")
            .update(pr_number)
            .labels(&labels)
            .send()
            .await?;

        Ok(())
    }

    /// Update PR title
    #[tracing::instrument]
    pub async fn update_pr_title(self, pr_number: u64, title: &str) -> anyhow::Result<()> {
        let Some(octocrab) = self.octocrab else {
            anyhow::bail!("GitHub client not initialized");
        };

        octocrab
            .issues("TUM-Dev", "NavigaTUM")
            .update(pr_number)
            .title(title)
            .send()
            .await?;

        Ok(())
    }

    /// Get the number of commits in a PR
    #[tracing::instrument]
    pub async fn get_pr_commit_count(self, pr_number: u64) -> anyhow::Result<usize> {
        let Some(octocrab) = self.octocrab else {
            anyhow::bail!("GitHub client not initialized");
        };

        // Fetch the first page of commits (up to 100 per page)
        let mut page = octocrab
            .pulls("TUM-Dev", "NavigaTUM")
            .pr_commits(pr_number)
            .per_page(100)
            .send()
            .await?;

        // Count commits from the first page
        let mut total_commits = page.items.len();

        // Follow pagination links to count commits from all subsequent pages
        while let Some(next_page) = octocrab.get_page(&page.next).await? {
            total_commits += next_page.items.len();
            page = next_page;
        }

        Ok(total_commits)
    }

    /// Get PR description (body)
    #[tracing::instrument]
    pub async fn get_pr_description(self, pr_number: u64) -> anyhow::Result<String> {
        let Some(octocrab) = self.octocrab else {
            anyhow::bail!("GitHub client not initialized");
        };

        let pr = octocrab
            .pulls("TUM-Dev", "NavigaTUM")
            .get(pr_number)
            .await?;

        Ok(pr.body.unwrap_or_default())
    }

    /// Update PR description (body)
    #[tracing::instrument(skip(description))]
    pub async fn update_pr_description(
        self,
        pr_number: u64,
        description: &str,
    ) -> anyhow::Result<()> {
        let Some(octocrab) = self.octocrab else {
            anyhow::bail!("GitHub client not initialized");
        };

        octocrab
            .issues("TUM-Dev", "NavigaTUM")
            .update(pr_number)
            .body(description)
            .send()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn newlines_whitespace() {
        assert_eq!(
            GitHub::clean_feedback_data("a\r\nb", 9),
            GitHub::clean_feedback_data("a\nb", 9)
        );
        assert_eq!(GitHub::clean_feedback_data("a\nb\nc", 9), "a  \nb  \nc");
        assert_eq!(GitHub::clean_feedback_data("a\nb  \nc", 9), "a  \nb  \nc");
        assert_eq!(GitHub::clean_feedback_data("a      \nb", 9), "a  \nb");
        assert_eq!(GitHub::clean_feedback_data("a\n\nb", 9), "a  \n  \nb");
        assert_eq!(GitHub::clean_feedback_data("a\n   b", 9), "a  \n   b");
    }
    #[test]
    fn truncate_len() {
        for i in 0..10 {
            let mut expected = "abcd".to_string();
            expected.truncate(i);
            assert_eq!(GitHub::clean_feedback_data("abcd", i), expected);
        }
    }
    #[test]
    fn special_cases() {
        assert_eq!(GitHub::clean_feedback_data("", 0), "");
        assert_eq!(GitHub::clean_feedback_data("a\x05bc", 9), "abc");
        assert_eq!(GitHub::clean_feedback_data("ab\x0Dc", 9), "abc");
    }
}
