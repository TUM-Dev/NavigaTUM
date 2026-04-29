use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::{Mutex, OnceCell};
use tracing::{debug, info, warn};

use crate::external::github::GitHub;

/// Inner state that exists only after the bare clone has completed.
struct Inner {
    bare_path: PathBuf,
    fetch_mutex: Mutex<()>,
    last_fetch: AtomicU64,
}

/// Manages a persistent bare clone of the `NavigaTUM` repository.
///
/// Instead of performing a full `git clone --depth=1` on every edit-proposal
/// request (~30s of network latency), we keep a bare repo around and use
/// `git worktree add` (~50ms, local-only) to create cheap, isolated working
/// directories for each request.
///
/// The initial bare clone is triggered from the background maintenance thread
/// after MS/DB setup.  If a request arrives before that, the `OnceCell` will
/// lazily perform the clone on demand.
pub struct RepoPool {
    inner: OnceCell<Inner>,
    /// Serializes the full edit→push cycle so two concurrent edits to the
    /// same batch branch cannot cause a non-fast-forward push failure.
    pub branch_mutex: Mutex<()>,
}

impl RepoPool {
    const FETCH_STALENESS_SECS: u64 = 60;

    /// Create a new `RepoPool`. This is cheap — the actual bare clone is
    /// deferred until `warm()` is called or the first `create_worktree()`.
    pub fn new() -> Self {
        Self {
            inner: OnceCell::new(),
            branch_mutex: Mutex::new(()),
        }
    }

    /// Eagerly trigger the bare clone so subsequent requests are fast.
    /// Safe to call multiple times — only the first call does work.
    pub async fn warm(&self) {
        match self.get_inner().await {
            Ok(_) => info!("RepoPool warmed up"),
            Err(e) => warn!(error = ?e, "RepoPool warm-up failed, will retry on first request"),
        }
    }

    /// Lazily initialise the bare repo on first use.
    async fn get_inner(&self) -> anyhow::Result<&Inner> {
        self.inner
            .get_or_try_init(|| async { Self::init_bare().await })
            .await
    }

    /// Bootstrap the bare repo.  Safe to call on every server start — it will
    /// either clone fresh or prune stale worktrees from a previous run.
    #[tracing::instrument]
    async fn init_bare() -> anyhow::Result<Inner> {
        let bare_path = env::temp_dir().join("navigatum-bare.git");

        if bare_path.exists() {
            // Validate existing repo
            let out = Command::new("git")
                .current_dir(&bare_path)
                .args(["rev-parse", "--is-bare-repository"])
                .output()
                .await?;
            let stdout = String::from_utf8_lossy(&out.stdout);
            if out.status.success() && stdout.trim() == "true" {
                info!(?bare_path, "Reusing existing bare repo");
                // Clean up stale worktrees from a previous server run
                let out = Command::new("git")
                    .current_dir(&bare_path)
                    .args(["worktree", "prune"])
                    .output()
                    .await?;
                debug!(output=?out, "git worktree prune");
            } else {
                warn!(?bare_path, "Corrupt bare repo, re-cloning");
                fs::remove_dir_all(&bare_path).await?;
                Self::clone_bare(&bare_path).await?;
            }
        } else {
            Self::clone_bare(&bare_path).await?;
        }

        // Ensure we can fetch all branches, not just the default one.
        let _ = Command::new("git")
            .current_dir(&bare_path)
            .args([
                "config",
                "remote.origin.fetch",
                "+refs/heads/*:refs/remotes/origin/*",
            ])
            .output()
            .await?;

        Ok(Inner {
            bare_path,
            fetch_mutex: Mutex::new(()),
            last_fetch: AtomicU64::new(0),
        })
    }

    async fn clone_bare(bare_path: &PathBuf) -> anyhow::Result<()> {
        let url = Self::authenticated_url()?;
        info!(?bare_path, "Cloning bare repo");
        let out = Command::new("git")
            .args(["clone", "--bare"])
            .arg(&url)
            .arg(bare_path)
            .output()
            .await
            .context("Failed to run git clone --bare")?;
        debug!(output=?out, "git clone --bare");
        if !out.status.success() {
            anyhow::bail!(
                "git clone --bare failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(())
    }

    fn authenticated_url() -> anyhow::Result<String> {
        let pat = GitHub::github_token().context("GITHUB_TOKEN must be set")?;
        Ok(format!("https://{pat}@github.com/TUM-Dev/NavigaTUM"))
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Fetch from origin if our local refs are stale (older than 60s).
    /// Also updates the remote URL in case the PAT has been rotated.
    #[tracing::instrument(skip(self))]
    async fn ensure_fresh(&self, branch: &str, is_new: bool) -> anyhow::Result<()> {
        let inner = self.get_inner().await?;
        let now = Self::now_secs();
        let last = inner.last_fetch.load(Ordering::Relaxed);
        if now.saturating_sub(last) < Self::FETCH_STALENESS_SECS {
            return Ok(());
        }

        let _guard = inner.fetch_mutex.lock().await;
        // Double-check after acquiring the lock — another task may have fetched.
        let last = inner.last_fetch.load(Ordering::Relaxed);
        if now.saturating_sub(last) < Self::FETCH_STALENESS_SECS {
            return Ok(());
        }

        // Update remote URL (handles PAT rotation)
        let url = Self::authenticated_url()?;
        let _ = Command::new("git")
            .current_dir(&inner.bare_path)
            .args(["remote", "set-url", "origin", &url])
            .output()
            .await?;

        // Fetch main (always needed)
        info!("Fetching origin");
        let out = Command::new("git")
            .current_dir(&inner.bare_path)
            .args(["fetch", "origin", "main"])
            .output()
            .await
            .context("Failed to fetch origin main")?;
        debug!(output=?out, "git fetch origin main");

        // For existing branches, also fetch the specific branch
        if !is_new {
            let out = Command::new("git")
                .current_dir(&inner.bare_path)
                .args(["fetch", "origin", branch])
                .output()
                .await
                .context("Failed to fetch branch")?;
            debug!(output=?out, branch, "git fetch origin <branch>");
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                // Only hard-fail if the data is really old
                if now.saturating_sub(last) > 300 {
                    anyhow::bail!("git fetch origin {branch} failed: {stderr}");
                }
                warn!(branch, %stderr, "fetch of branch failed, continuing with stale data");
            }
        }

        inner.last_fetch.store(now, Ordering::Relaxed);
        Ok(())
    }

    /// Create an isolated worktree for a single edit-proposal request.
    #[tracing::instrument(skip(self))]
    pub async fn create_worktree(
        &self,
        branch: &str,
        is_new: bool,
    ) -> anyhow::Result<super::tmp_repo::Worktree> {
        self.ensure_fresh(branch, is_new).await?;
        let inner = self.get_inner().await?;

        let dir = tempfile::tempdir()?;
        let dir_path = dir.path().to_string_lossy().to_string();

        if is_new {
            // New branch from origin/main
            let out = Command::new("git")
                .current_dir(&inner.bare_path)
                .args(["worktree", "add", &dir_path, "-b", branch, "origin/main"])
                .output()
                .await
                .context("git worktree add (new branch)")?;
            debug!(output=?out, "git worktree add -b");
            if !out.status.success() {
                anyhow::bail!(
                    "git worktree add (new branch) failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        } else {
            // Existing branch — check it out from the remote tracking ref
            let remote_ref = format!("origin/{branch}");
            let out = Command::new("git")
                .current_dir(&inner.bare_path)
                .args(["worktree", "add", &dir_path, &remote_ref])
                .output()
                .await
                .context("git worktree add (existing branch)")?;
            debug!(output=?out, "git worktree add (existing)");
            if !out.status.success() {
                anyhow::bail!(
                    "git worktree add (existing branch) failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }

            // Point the local branch at the remote tip
            let out = Command::new("git")
                .current_dir(dir.path())
                .args(["checkout", "-B", branch, &remote_ref])
                .output()
                .await
                .context("git checkout -B")?;
            debug!(output=?out, "git checkout -B");
            if !out.status.success() {
                anyhow::bail!(
                    "git checkout -B failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }

        // Configure committer identity
        let _ = Command::new("git")
            .current_dir(dir.path())
            .args(["config", "user.email", "actions@github.com"])
            .output()
            .await?;
        let _ = Command::new("git")
            .current_dir(dir.path())
            .args(["config", "user.name", "GitHub Actions"])
            .output()
            .await?;

        // Set the push URL (so `git push` uses the current PAT)
        let url = Self::authenticated_url()?;
        let _ = Command::new("git")
            .current_dir(dir.path())
            .args(["remote", "set-url", "origin", &url])
            .output()
            .await?;

        info!(worktree = %dir_path, branch, is_new, "Created worktree");
        Ok(super::tmp_repo::Worktree {
            dir,
            branch_name: branch.to_string(),
            bare_path: inner.bare_path.clone(),
        })
    }
}
