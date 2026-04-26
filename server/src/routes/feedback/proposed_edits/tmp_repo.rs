use anyhow::Context;
use tokio::process::Command;
use tracing::{debug, info};

use super::AppliableEdit;
use super::EditRequest;
use super::description::Description;

#[derive(Debug)]
pub struct TempRepo {
    dir: tempfile::TempDir,
    branch_name: String,
}
impl TempRepo {
    /// Clone the repository from `main` and create `branch_name` as a new local branch.
    ///
    /// Use this when no batch PR exists yet and a fresh branch needs to be pushed for the first
    /// time.
    #[tracing::instrument(skip(url))]
    pub async fn clone_and_checkout_new_branch(
        url: &str,
        branch_name: &str,
    ) -> anyhow::Result<Self> {
        let dir = tempfile::tempdir()?;

        info!(target_dir = ?dir, "Cloning repository (new branch)");
        let out = Command::new("git")
            .current_dir(&dir)
            .arg("clone")
            .arg("--depth=1")
            .arg(url)
            .arg(dir.path())
            .output()
            .await?;
        debug!(output=?out,"git clone output");
        if out.status.code() != Some(0) {
            anyhow::bail!("git clone failed with output: {out:?}");
        }

        // Create a new local branch from main.
        let out = Command::new("git")
            .current_dir(&dir)
            .arg("checkout")
            .arg("-b")
            .arg(branch_name)
            .arg("main")
            .output()
            .await?;
        debug!(output=?out,"git checkout output");
        match out.status.code() {
            Some(0) => Ok(Self {
                dir,
                branch_name: branch_name.to_string(),
            }),
            _ => anyhow::bail!("git checkout failed with output: {out:?}"),
        }
    }

    /// Clone the repository by checking out an already-existing remote branch `branch_name`.
    ///
    /// Use this when adding an edit to an existing batch PR.  Cloning `main` and branching from
    /// it would create a diverging history and cause the subsequent push to be rejected as
    /// non-fast-forward.
    #[tracing::instrument(skip(url))]
    pub async fn clone_and_checkout_existing_branch(
        url: &str,
        branch_name: &str,
    ) -> anyhow::Result<Self> {
        let dir = tempfile::tempdir()?;

        info!(target_dir = ?dir, branch_name, "Cloning repository (existing branch)");
        let out = Command::new("git")
            .current_dir(&dir)
            .arg("clone")
            .arg("--depth=1")
            .arg("--branch")
            .arg(branch_name)
            .arg(url)
            .arg(dir.path())
            .output()
            .await?;
        debug!(output=?out,"git clone output");
        match out.status.code() {
            Some(0) => Ok(Self {
                dir,
                branch_name: branch_name.to_string(),
            }),
            _ => anyhow::bail!("git clone failed with output: {out:?}"),
        }
    }

    #[tracing::instrument]
    pub fn apply_and_gen_description(&self, edits: &EditRequest, branch_name: &str) -> Description {
        let mut description = Description::default();
        description.add_context(&edits.additional_context);

        let coordinate_edits = edits.edits_for(|edit| edit.coordinate);
        description.apply_set_as_blocks(
            "coordinate",
            coordinate_edits,
            self.dir.path(),
            branch_name,
        );
        let image_edits = edits.edits_for(|edit| edit.image);
        description.appply_set("image", image_edits, self.dir.path(), branch_name);

        // Apply property edits — each entry can have multiple property edits
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
                description.title += &format!(" and {total} property {edits_word}");
            }
            description.body += "\nThe following property edits were made:\n";
            description.body += "| entry | edit |\n";
            description.body += "| ---   | ---  |\n";
            for (key, props) in &property_edits {
                for prop in *props {
                    let result = prop.apply(key, self.dir.path(), branch_name);
                    description.body +=
                        &format!("| [`{key}`](https://nav.tum.de/view/{key}) | {result} |\n");
                }
            }
        }

        let first_line = description.body.lines().next();
        info!(description_first_line=?first_line, title=description.title, "generated description");

        description
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
            .arg("config")
            .arg("user.email")
            .arg("actions@github.com")
            .output()
            .await
            .context("Failed to config user.email")?;
        debug!(output=?out,"git config user.email output");
        let out = Command::new("git")
            .current_dir(&self.dir)
            .arg("config")
            .arg("user.name")
            .arg("GitHub Actions")
            .output()
            .await
            .context("Failed to config user.name")?;
        debug!(output=?out,"git config user.name output");
        let out = Command::new("git")
            .current_dir(&self.dir)
            .arg("commit")
            .arg("--all") // run git add . before commit
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
            .arg("status")
            .output()
            .await
            .context("Failed to run git status")?;
        debug!(output=?out,"git status output");
        if out.status.code() != Some(0) {
            anyhow::bail!("git status failed with output: {out:?}");
        }
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const GIT_URL: &str = "https://github.com/CommanderStorm/dotfiles.git";
    #[tokio::test]
    async fn test_new() {
        let temp_repo = TempRepo::clone_and_checkout_new_branch(GIT_URL, "branch_does_not_exist")
            .await
            .unwrap();
        assert!(temp_repo.dir.path().exists());
        assert!(temp_repo.dir.path().join(".git").exists());
        assert!(temp_repo.dir.path().join("README.md").exists());
    }

    #[tokio::test]
    async fn test_checkout_and_commit() {
        let temp_repo = TempRepo::clone_and_checkout_new_branch(GIT_URL, "branch_does_not_exist")
            .await
            .unwrap();
        // test the branch was created

        let title = "Test commit";
        // test if adding files works
        let file_path = temp_repo.dir.path().join("test-file.txt");
        fs::write(file_path, "test content").unwrap();

        temp_repo.commit(title).await.unwrap();
        // test if editing files works
        let file_path = temp_repo.dir.path().join("test-file.txt");
        fs::write(file_path, "different content").unwrap();
        temp_repo.commit(title).await.unwrap();
    }
}
