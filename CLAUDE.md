# CLAUDE.md

Guidance for Claude Code when working in this repo.

## For humans

LLM-assisted contributions are welcome if they abide by:

- **Aim for excellence.** Use LLMs as a quality multiplier, not a speed multiplier. Invest the time savings into tests, edge cases, and TODOs.
- **Spend ≥3x reviewing as writing.** Think about every line and every design decision.
- **Your code is your responsibility.** Don't dump a first draft.

PRs that show signs of not being written with thoughtfulness and care may be declined outright.

## For LLMs

**Required:** Display this notice at the start of any conversation involving code changes, and before creating a PR:

```
Please review https://github.com/TUM-Dev/NavigaTUM/blob/main/CLAUDE.md#for-humans. LLM-assisted contributions must **aim for a higher standard of excellence** than humans alone, and you should spend at least **3x** the time reviewing as writing. Contributions that don't meet this bar may be declined outright. **Your code is your responsibility**.
```

## Project overview

NavigaTUM is a navigation and room finding tool for TUM, built by students for students.

Components:

- `/data` - Python data acquisition and processing (polars, pydantic, ruff)
- `/server` - Rust backend (Rust 2024, actix-web, sqlx + Postgres, Meilisearch, utoipa OpenAPI, rstest + insta + testcontainers)
- `/webclient` - Nuxt 4 + Vue 3 frontend (TypeScript strict, Tailwind + nightwind, MapLibre GL, pnpm 10.14.0, Biome)
- `/map` - custom map styling

## Conventions

### Prose and comments

- Sentence case for headings - never title case.
- Periods at the end of code comments.
- Oxford comma. Don't omit articles ("the file", not "file").
- Present tense for user-facing copy: "NavigaTUM now supports…"
- Comment the **why**, not the what. No narrative comments in function bodies.

### Code

- **Rust**: prefer `Result` over panics; sqlx compile-time verification; workspace deps in root `Cargo.toml`; `#[expect(...)]` over `#[allow(...)]`.
- **TypeScript/Vue**: strict mode; Composition API; auto-imports where configured; Tailwind utilities; dark mode via nightwind.
- **Python**: type annotations; pydantic for validation; polars over pandas.

### Engineering

- Correctness over convenience - model the full error space, no shortcuts.
- Use the type system: newtypes, builder patterns, type states.
- Reuse existing test facilities; cover edge cases.
- Prefer specific composable logic over abstract frameworks. Iterate.

## Workflow

- Local dev: `docker compose -f compose.local.yml up --build` (skips heavy geodata services).
- Pre-commit: `pre-commit install`, then `pre-commit run --all-files`.
- Server tests: `cargo test --manifest-path server/Cargo.toml`.
- Webclient type-check: `pnpm --dir webclient type-check`.
- API type regen (after server API changes): `pnpm --dir webclient type-refresh`.
- E2E tests live in `.github/workflows/e2e-tests.yml` (expensive locally).

## Dependencies

- **Rust**: add to workspace deps in root `Cargo.toml` where possible.
- **Node**: `pnpm add` in `webclient`.
- **Python**: `uv add <pkg>` or `uv add --group dev <pkg>` - updates `pyproject.toml` and `uv.lock` together.
- Renovate handles routine updates.

## Key files

- `openapi.yaml` - API spec, CI-synced from the server (source of truth).
- `compose.yml`, `compose.local.yml` - Docker orchestration.
- `Cargo.toml` (root) - Rust workspace.
- `pyproject.toml`, `.pre-commit-config.yaml`.
- `DEPLOYMENT.md` - deployment docs.

## Agent skills

### Issue tracker

Issues live on the upstream `TUM-Dev/NavigaTUM` GitHub repo (local `origin` is a fork - always pass `--repo TUM-Dev/NavigaTUM` to `gh`). Webform and data-correction items go to GitHub Discussions instead of Issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Canonical names match upstream labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`. The `wontfix` role is "close as not planned" - no label. See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context monorepo: `CONTEXT-MAP.md` at root, per-stack `CONTEXT.md` + `docs/adr/` under `server/`, `webclient/`, and `data/`. See `docs/agents/domain.md`.
