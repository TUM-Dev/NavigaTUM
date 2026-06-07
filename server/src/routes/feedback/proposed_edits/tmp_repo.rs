use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use tokio::process::Command;
use tracing::{debug, info};

use super::AppliableEdit as _;
use super::EditRequest;
use super::description::Description;

#[derive(Debug)]
pub struct Worktree {
    pub(super) dir: tempfile::TempDir,
    pub(super) branch_name: String,
    pub(super) bare_path: PathBuf,
}

impl Worktree {
    pub fn base_dir(&self) -> &Path {
        self.dir.path()
    }

    #[tracing::instrument]
    pub fn apply_and_gen_description(
        &self,
        edits: &EditRequest,
        branch_name: &str,
    ) -> anyhow::Result<Description> {
        let mut description = Description::default();
        description.add_context(&edits.additional_context);

        // Additions go before edits so that an edit in the same request can target a
        // newly-added entry (e.g. add a room, then edit its coordinate). The reverse order
        // would reject the edit because the target wouldn't exist yet.
        description.apply_additions(&edits.additions, self.dir.path(), branch_name)?;

        let coordinate_edits = edits.edits_for(|edit| edit.coordinate);
        description.apply_set_as_blocks(
            "coordinate",
            coordinate_edits,
            self.dir.path(),
            branch_name,
        )?;
        let image_edits = edits.edits_for(|edit| edit.image);
        description.apply_set("image", image_edits, self.dir.path(), branch_name)?;

        // Apply property edits - each entry can have multiple property edits
        let property_edits: Vec<(&str, &[super::property::PropertyEdit])> = edits
            .edits
            .0
            .iter()
            .filter_map(|(k, edit)| {
                edit.properties
                    .as_deref()
                    .filter(|p| !p.is_empty())
                    .map(|p| (k.as_str(), p))
            })
            .collect();
        if !property_edits.is_empty() {
            let total: usize = property_edits.iter().map(|(_, v)| v.len()).sum();
            let edits_word = if total == 1 { "edit" } else { "edits" };
            if description.title.is_empty() {
                description.title = format!("{total} property {edits_word}");
            } else {
                write!(description.title, " and {total} property {edits_word}")?;
            }
            description.body += "\nThe following property edits were made:\n";
            description.body += "| entry | edit |\n";
            description.body += "| ---   | ---  |\n";
            for (key, props) in &property_edits {
                for prop in *props {
                    let result = prop.apply(key, self.dir.path(), branch_name)?;
                    writeln!(
                        description.body,
                        "| [`{key}`](https://nav.tum.de/view/{key}) | {result} |"
                    )?;
                }
            }
        }

        let first_line = description.body.lines().next();
        info!(description_first_line=?first_line, title=description.title, "generated description");

        Ok(description)
    }

    #[tracing::instrument]
    pub async fn commit(&self, title: &str) -> anyhow::Result<()> {
        info!(title, "Commiting changes");

        let out = Command::new("git")
            .current_dir(&self.dir)
            .arg("add")
            .arg(".")
            .output()
            .await
            .context("Failed to add files to git")?;
        debug!(output=?out,"git add output");
        let out = Command::new("git")
            .current_dir(&self.dir)
            .arg("commit")
            .arg("--all")
            .arg("--message")
            .arg(title)
            .output()
            .await
            .context("Failed to commit changes")?;
        debug!(output=?out,"git commit output");
        match out.status.code() {
            Some(0) => Ok(()),
            _ => anyhow::bail!("git commit failed with output: {out:?}"),
        }
    }

    #[tracing::instrument]
    pub async fn push(&self) -> anyhow::Result<()> {
        info!("Pushing changes to the remote");
        let out = Command::new("git")
            .current_dir(&self.dir)
            .arg("push")
            .arg("--set-upstream")
            .arg("origin")
            .arg(&self.branch_name)
            .output()
            .await
            .context("Failed to push to upstream")?;
        debug!(output=?out,"git push output");
        match out.status.code() {
            Some(0) => Ok(()),
            _ => anyhow::bail!("git push failed with output: {out:?}"),
        }
    }
}

impl Drop for Worktree {
    fn drop(&mut self) {
        let bare_path = self.bare_path.clone();
        let dir_path = self.dir.path().to_path_buf();
        // Best-effort cleanup - fire and forget.
        tokio::spawn(async move {
            let path_str = dir_path.to_string_lossy().to_string();
            if let Err(e) = Command::new("git")
                .current_dir(&bare_path)
                .args(["worktree", "remove", "--force", &path_str])
                .output()
                .await
            {
                debug!(error = ?e, "best-effort worktree removal failed to spawn");
            }
            if let Err(e) = Command::new("git")
                .current_dir(&bare_path)
                .args(["worktree", "prune"])
                .output()
                .await
            {
                debug!(error = ?e, "best-effort worktree prune failed to spawn");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::let_underscore_must_use,
        reason = "tests assert via panic/unwrap and ignore best-effort git command results"
    )]
    use std::fs;

    use super::*;

    const GIT_URL: &str = "https://github.com/CommanderStorm/dotfiles.git";

    /// Helper: create a Worktree the old-fashioned way (via clone) for tests
    /// that don't have a bare repo available.
    async fn clone_worktree(url: &str, branch_name: &str) -> anyhow::Result<Worktree> {
        let dir = tempfile::tempdir()?;

        let out = Command::new("git")
            .current_dir(&dir)
            .args(["clone", "--depth=1", url])
            .arg(dir.path())
            .output()
            .await?;
        if !out.status.success() {
            anyhow::bail!("git clone failed: {out:?}");
        }

        let out = Command::new("git")
            .current_dir(&dir)
            .args(["checkout", "-b", branch_name, "main"])
            .output()
            .await?;
        if !out.status.success() {
            anyhow::bail!("git checkout failed: {out:?}");
        }

        let _ = Command::new("git")
            .current_dir(&dir)
            .args(["config", "user.email", "test@test.com"])
            .output()
            .await;
        let _ = Command::new("git")
            .current_dir(&dir)
            .args(["config", "user.name", "Test"])
            .output()
            .await;

        Ok(Worktree {
            bare_path: dir.path().to_path_buf(),
            dir,
            branch_name: branch_name.to_string(),
        })
    }

    #[tokio::test]
    async fn test_new() {
        let worktree = clone_worktree(GIT_URL, "branch_does_not_exist")
            .await
            .unwrap();
        assert!(worktree.dir.path().exists());
        assert!(worktree.dir.path().join(".git").exists());
        assert!(worktree.dir.path().join("README.md").exists());
    }

    #[tokio::test]
    async fn test_checkout_and_commit() {
        let worktree = clone_worktree(GIT_URL, "branch_does_not_exist")
            .await
            .unwrap();

        let title = "Test commit";
        let file_path = worktree.dir.path().join("test-file.txt");
        fs::write(&file_path, "test content").unwrap();
        worktree.commit(title).await.unwrap();

        fs::write(&file_path, "different content").unwrap();
        worktree.commit(title).await.unwrap();
    }
}
