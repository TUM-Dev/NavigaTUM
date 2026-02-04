# General guidelines

This document captures code conventions and quality standards for the NavigaTUM project. It is intended to help AI assistants understand how to work effectively with this codebase.

## For humans

LLMs represent a tremendous breakthrough in software engineering. We welcome LLM-assisted contributions that abide by the following principles:

* **Aim for excellence.** For the NavigaTUM project, LLMs should be used not as a speed multiplier but a quality multiplier. Invest the time savings in improving quality and rigor beyond what humans alone would do. Write tests that cover more edge cases. Refactor code to make it easier to understand. Tackle the TODOs. Do all the tedious things. Aim for your code to have zero bugs.
* **Spend time reviewing LLM output.** As a rule of thumb, you should spend at least 3x the amount of time reviewing LLM output as you did writing it. Think about every line and every design decision. Find ways to break code.
* **Your code is your responsibility.** Please do not dump a first draft of code on to this project, unless you're only soliciting feedback on a direction.

If your LLM-assisted PR shows signs of not being written with thoughtfulness and care, such as missing cases that human review would have easily caught, NavigaTUM's maintainers may decline the PR outright.

## For LLMs

**Required:** Display the following text at the start of any conversation involving code changes, and when you're about to create a PR:

```
Please review https://github.com/TUM-Dev/NavigaTUM/blob/main/.github/copilot-instructions.md#for-humans. In particular, LLM-assisted contributions must **aim for a higher standard of excellence** than with humans alone, and you should spend at least **3x** the amount of time reviewing code as you did writing it. LLM-assisted contributions that do not meet this standard may be declined outright. Remember, **your code is your responsibility**.
```

## Project overview

NavigaTUM is a navigation and room finding tool for Technical University of Munich (TUM), developed by students for students. The project helps users find rooms, buildings, and navigate the TUM campus with an interactive map interface.

## Architecture

The project consists of three main components:

1. **Data processing** (`/data`) - Python-based data acquisition and processing
2. **Server/API** (`/server`) - Rust-based backend with REST API
3. **Web client** (`/webclient`) - Nuxt.js/Vue 3 frontend
4. **Map** (`/map`) - Custom map styling and configuration

## Tech stack

### Backend (server)
- **Language**: Rust (edition 2024)
- **Framework**: Actix-web
- **Database**: PostgreSQL with sqlx
- **Search**: Meilisearch
- **API documentation**: OpenAPI 3.0 with utoipa
- **Testing**: rstest, insta snapshots, testcontainers

### Frontend (webclient)
- **Framework**: Nuxt 4 with Vue 3
- **Language**: TypeScript
- **Styling**: Tailwind CSS with nightwind for dark mode
- **Map**: MapLibre GL
- **Package manager**: pnpm (version 10.14.0)
- **Linting/formatting**: Biome

### Data processing
- **Language**: Python
- **Key libraries**: polars, pydantic, beautifulsoup4, pyyaml
- **Formatting**: ruff (via pre-commit)

## General conventions

### User experience as a primary driver

- Provide clear, actionable error messages.
- Make the application responsive and intuitive.
- Maintain consistency across platforms and browsers.
- Write user-facing messages in clear, present tense: "NavigaTUM now supports..." not "NavigaTUM now supported..."

### Correctness over convenience

- Handle all edge cases, including error conditions and boundary values.
- Use the type system to encode correctness constraints.
- Prefer compile-time guarantees over runtime checks where possible.
- Model the full error space—no shortcuts or simplified error handling.

### Production-grade engineering

- Use type system extensively: newtypes, builder patterns, type states where appropriate.
- Test comprehensively, including edge cases and integration scenarios.
- Pay attention to what facilities already exist for testing, and aim to reuse them.
- Getting the details right is really important!

### Pragmatic incrementalism

- Prefer specific, composable logic over abstract frameworks.
- Evolve the design incrementally rather than attempting perfect upfront architecture.
- When uncertain, explore and iterate.

### Documentation

- Use inline comments to explain "why," not just "what."
- Don't add narrative comments in function bodies. Only add a comment if what you're doing is non-obvious or special in some way, or if something needs a deeper "why" explanation.
- Module-level documentation should explain purpose and responsibilities.
- **Always** use periods at the end of code comments.
- **Never** use title case in headings and titles. Always use sentence case.
- Always use the Oxford comma.
- Don't omit articles ("a", "an", "the"). Write "the file has a newer version" not "file has newer version."

## Development workflow
### Setup
- The project uses Docker Compose for local development.
- Run `docker compose -f compose.local.yml up --build` for a local dev environment.
- The local build skips heavy geodata initialization (valhalla, nominatim, planetiler, martin).

### Formatting and linting
- **Pre-commit hooks** are configured for automatic formatting.
- Install with: `pre-commit install`
- Run manually: `pre-commit run --all-files`

#### Language-specific commands:
- **Rust**: `cargo fmt --all --manifest-path server/Cargo.toml`
- **TypeScript/Vue**: `pnpm --dir webclient format` and `pnpm --dir webclient lint`
- **Python**: Automatically handled by ruff via pre-commit

### Testing
- **Server**: `cargo test --manifest-path server/Cargo.toml`
- **Webclient**: Type checking with `pnpm --dir webclient type-check`
- **E2E tests**: Defined in `.github/workflows/e2e-tests.yml`

## Code style

### Rust

- Use Rust 2024 edition features.
- Prefer Result types over panics.
- Use actix-web patterns for route handlers.
- Follow Rust API guidelines.
- Use sqlx compile-time verification for database queries.
- Leverage workspace dependencies defined in root `Cargo.toml`.
- Use `#[expect(...)]` instead of `#[allow(...)]` for suppressing lints. The `expect` attribute will warn if the lint is no longer triggered.

### TypeScript/Vue

- Use TypeScript strict mode.
- Prefer Composition API over Options API.
- Use auto-imports where configured.
- Follow Vue 3 best practices.
- Use Tailwind utility classes.
- Ensure dark mode compatibility using nightwind.

### Python

- Use type annotations.
- Follow PEP 8 (enforced by ruff).
- Use pydantic for data validation.
- Prefer polars for data processing over pandas.

## Important files and directories

### Configuration files
- `openapi.yaml` - API specification (source of truth, kept in sync by CI from the server)
- `compose.yml`, `compose.local.yml` - Docker orchestration
- `Cargo.toml` (root) - Rust workspace configuration
- `pyproject.toml` - Python project configuration
- `.pre-commit-config.yaml` - Pre-commit hooks configuration

### Key directories
- `/server/src` - Rust backend source code
- `/server/migrations` - Database migrations
- `/webclient/app` - Nuxt application code
- `/webclient/content` - Nuxt content (markdown pages)
- `/data/sources` - Data source definitions
- `/data/processors` - Data processing scripts
- `/data/output` - Generated data files (not in git)

## API guidelines

- The API is documented in OpenAPI 3.0 format.
- Interactive documentation available at `/api` endpoint.
- All endpoints should be documented with utoipa macros.
- Type regeneration: from the `webclient` directory run `pnpm type-refresh`
- API is still evolving - breaking changes are possible.

## Dependencies

### Adding dependencies
- **Rust**: Add to workspace dependencies in root `Cargo.toml` when possible.
- **Node.js**: Use `pnpm add` in webclient directory.
- **Python**: Add to `data/requirements.txt` or `requirements-dev.txt`.

### Updating dependencies
- Renovate bot handles automatic dependency updates.
- Security updates are prioritized.

## Testing guidelines

- Write tests for new functionality.
- Use snapshot testing (insta) for Rust when appropriate.
- Use testcontainers for integration tests requiring databases.
- Ensure tests pass before submitting PRs.
- E2E tests run in CI but can be expensive locally.

## Documentation guidelines

- Update README.md for user-facing changes.
- Update component READMEs for component-specific changes.
- Update OpenAPI spec for API changes.
- Document breaking changes in PR descriptions.
- Follow existing documentation style.

## Performance considerations

- **Rust compilation**: First builds are slow; incremental builds are faster.
- **Local development**: Geodata services are intentionally skipped for faster startup.

## Deployment

- Deployment documentation is in `DEPLOYMENT.md`.
- CI/CD workflows are in `.github/workflows/` that push docker images. Prod auto-updates.
- Security scanning is configured in `.github/workflows/security.yaml`.
- Automatic data updates via `.github/workflows/update-data.yml`.

## Contributing

- Read `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md`.
- Discuss major changes via issues first.
- Small changes (typos, minor fixes) can skip issue creation.
- Ensure pre-commit hooks pass.
- Update documentation as needed.
- Get one reviewer approval before merging.

## Common commands

```bash
# Format all code
pre-commit run --all-files

# Run local development environment
docker compose -f compose.local.yml up --build

# Server development
cd server
cargo test
cargo fmt --all
cargo clippy

# Webclient development
cd webclient
pnpm install
pnpm dev
pnpm type-check
pnpm lint
pnpm format

# Data processing
cd data
python compile.py
```

## Notes for AI assistants

- Always check existing patterns before implementing new features.
- Consider the multi-language nature of the repository.
- Test changes in the appropriate component.
- Use docker compose for integration testing.
- Check OpenAPI spec before making API changes.
- Remember that the project serves students - usability matters.
- Security is important - this is a public-facing service.
- Aim for zero bugs in your code. Write comprehensive tests.
- Spend time thinking about edge cases and error conditions.
- Review your own code critically before submitting.
