use actix_web::HttpResponse;
use octocrab::Octocrab;
use regex::Regex;
use tracing::error;

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

        let resp = octocrab
            .issues("TUM-Dev", "navigatum")
            .create(title)
            .body(description)
            .labels(labels)
            .send()
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
    ) -> HttpResponse {
        let Some(octocrab) = self.octocrab else {
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed to create a pull request, please try again later");
        };

        // create the PR
        let pr_number = match octocrab
            .pulls("TUM-Dev", "NavigaTUM")
            .create(title, branch, "main")
            .body(description)
            .maintainer_can_modify(true)
            .send()
            .await
        {
            Ok(pr) => pr.number,
            Err(e) => {
                error!(error = ?e, "Error creating pull request");
                return HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("Failed to create a pull request, please try again later");
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
            Ok(issue) => HttpResponse::Created()
                .content_type("text/plain")
                .body(issue.html_url.to_string()),
            Err(e) => {
                error!(error = ?e, "Error updating PR");
                HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("Failed to create a pull request, please try again later")
            }
        }
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

        let re = Regex::new(r"[ \t]*\n").unwrap();
        re.replace_all(&s_clean, "  \n").to_string()
    }

    pub fn github_token() -> Option<String> {
        match std::env::var("GITHUB_TOKEN") {
            Ok(token) => Some(token.trim().to_string()),
            Err(e) => {
                error!(error = ?e, "GITHUB_TOKEN has to be set for feedback");
                None
            }
        }
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
