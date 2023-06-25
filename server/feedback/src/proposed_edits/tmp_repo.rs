use crate::proposed_edits::EditRequest;
use log::{debug, info};
use std::error;
use tokio::process::Command;

use crate::proposed_edits::discription::Description;

pub struct TempRepo {
    dir: tempfile::TempDir,
    branch_name: String,
}
impl TempRepo {
    pub async fn clone_and_checkout(
        url: &'static str,
        branch_name: &str,
    ) -> Result<Self, Box<dyn error::Error>> {
        let dir = tempfile::tempdir()?;

        info!("Cloning {url} into {dir:?}");
        let out = Command::new("git")
            .current_dir(&dir)
            .arg("clone")
            .arg(url)
            .arg(dir.path())
            .output()
            .await?;
        debug!("commit output: {out:?}");
        if out.status.code() != Some(0) {
            return Err(format!("git status failed with output: {out:?}").into());
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
            _ => Err(format!("git commit failed with output: {out:?}").into()),
        }
    }

    pub fn apply_and_gen_description(&self, edits: &EditRequest) -> Description {
        let mut description = Description::default();
        description.add_context(&edits.additional_context);

        let coordinate_edits = edits.edits_for(|edit| edit.coordinate);
        description.appply_set("coordinate", coordinate_edits, self.dir.path());
        let image_edits = edits.edits_for(|edit| edit.image);
        description.appply_set("image", image_edits, self.dir.path());

        description
    }

    pub async fn commit(&self, title: &str) -> Result<(), Box<dyn error::Error>> {
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
            _ => Err(format!("git commit failed with output: {out:?}").into()),
        }
    }
    pub async fn push(&self) -> Result<(), Box<dyn error::Error>> {
        let out = Command::new("git")
            .current_dir(&self.dir)
            .arg("status")
            .output()
            .await?;
        debug!("git status: {out:?}");
        if out.status.code() != Some(0) {
            return Err(format!("git status failed with output: {out:?}").into());
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
            _ => Err(format!("git push failed with output: {out:?}").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    const GIT_URL: &str = "https://github.com/CommanderStorm/dotfiles.git";
    #[tokio::test]
    async fn test_new() {
        let _ = env_logger::builder().is_test(true).try_init();
        let temp_repo = TempRepo::clone_and_checkout(GIT_URL, "main").await.unwrap();
        assert!(temp_repo.dir.path().exists());
        assert!(temp_repo.dir.path().join(".git").exists());
        assert!(temp_repo.dir.path().join("README.md").exists());
    }

    #[tokio::test]
    async fn test_checkout_and_commit() {
        let _ = env_logger::builder().is_test(true).try_init();
        let temp_repo = TempRepo::clone_and_checkout(GIT_URL, "branch_does_not_exist")
            .await
            .unwrap();
        // test the branch was created

        // test the commit
        let title = "Test commit";
        let file_path = temp_repo.dir.path().join("test-file.txt");
        fs::write(file_path, "test content").unwrap();

        temp_repo.commit(title).await.unwrap();
    }
}
