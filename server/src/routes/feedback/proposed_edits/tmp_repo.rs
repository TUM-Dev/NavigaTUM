use tokio::process::Command;
use tracing::{debug, info};

use super::discription::Description;
use super::EditRequest;

#[derive(Debug)]
pub struct TempRepo {
    dir: tempfile::TempDir,
    branch_name: String,
}
impl TempRepo {
    #[tracing::instrument]
    pub async fn clone_and_checkout(url: &'static str, branch_name: &str) -> anyhow::Result<Self> {
        let dir = tempfile::tempdir()?;

        info!("Cloning {url} into {dir:?}");
        let out = Command::new("git")
            .current_dir(&dir)
            .arg("clone")
            .arg("--depth=1")
            .arg(url)
            .arg(dir.path())
            .output()
            .await?;
        debug!("commit output: {out:?}");
        if out.status.code() != Some(0) {
            anyhow::bail!("git status failed with output: {out:?}");
        }

        // checkout + create branch
        let out = Command::new("git")
            .current_dir(&dir)
            .arg("checkout")
            .arg("-b")
            .arg(branch_name)
            .arg("main")
            .output()
            .await?;
        debug!("checkout output: {out:?}");
        match out.status.code() {
            Some(0) => Ok(Self {
                dir,
                branch_name: branch_name.to_string(),
            }),
            _ => anyhow::bail!("git commit failed with output: {out:?}"),
        }
    }

    #[tracing::instrument]
    pub fn apply_and_gen_description(&self, edits: &EditRequest) -> Description {
        let mut description = Description::default();
        description.add_context(&edits.additional_context);

        let coordinate_edits = edits.edits_for(|edit| edit.coordinate);
        description.appply_set("coordinate", coordinate_edits, self.dir.path());
        let image_edits = edits.edits_for(|edit| edit.image);
        description.appply_set("image", image_edits, self.dir.path());

        description
    }

    #[tracing::instrument]
    pub async fn commit(&self, title: &str) -> anyhow::Result<()> {
        let out = Command::new("git")
            .current_dir(&self.dir)
            .arg("add")
            .arg(".")
            .output()
            .await?;
        debug!("git-add output: {out:?}");
        let out = Command::new("git")
            .current_dir(&self.dir)
            .arg("commit")
            .arg("--all") // run git add . before commit
            .arg("-m")
            .arg(title)
            .output()
            .await?;
        debug!("commit output: {out:?}");
        match out.status.code() {
            Some(0) => Ok(()),
            _ => anyhow::bail!("git commit failed with output: {out:?}"),
        }
    }
    #[tracing::instrument]
    pub async fn push(&self) -> anyhow::Result<()> {
        let out = Command::new("git")
            .current_dir(&self.dir)
            .arg("status")
            .output()
            .await?;
        debug!("git status: {out:?}");
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
            .await?;
        debug!("git push: {out:?}");
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
        let temp_repo = TempRepo::clone_and_checkout(GIT_URL, "branch_does_not_exist")
            .await
            .unwrap();
        assert!(temp_repo.dir.path().exists());
        assert!(temp_repo.dir.path().join(".git").exists());
        assert!(temp_repo.dir.path().join("README.md").exists());
    }

    #[tokio::test]
    async fn test_checkout_and_commit() {
        let temp_repo = TempRepo::clone_and_checkout(GIT_URL, "branch_does_not_exist")
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
