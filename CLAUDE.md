# CLAUDE.md

Guidance for Claude Code when working in this repo.

## Agent skills

### Issue tracker

Issues live on the upstream `TUM-Dev/NavigaTUM` GitHub repo (local `origin` is a fork — always pass `--repo TUM-Dev/NavigaTUM` to `gh`). Webform and data-correction items go to GitHub Discussions instead of Issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Canonical names match upstream labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`. The `wontfix` role is "close as not planned" — no label. See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context monorepo: `CONTEXT-MAP.md` at root, per-stack `CONTEXT.md` + `docs/adr/` under `server/`, `webclient/`, and `data/`. See `docs/agents/domain.md`.
