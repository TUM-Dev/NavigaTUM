# GitHub Copilot Instructions for NavigaTUM

## Project Overview

NavigaTUM is a navigation and room finding tool for Technical University of Munich (TUM), developed by students for students.
The project helps users find rooms, buildings, and navigate the TUM campus with an interactive map interface.

## Architecture

The project consists of three main components:

1. **Data Processing** (`/data`) - Python-based data acquisition and processing
2. **Server/API** (`/server`) - Rust-based backend with REST API
3. **Web Client** (`/webclient`) - Nuxt.js/Vue 3 frontend
4. **Map** (`/map`) - Custom map styling and configuration

## Tech Stack

### Backend (Server)
- **Language**: Rust (Edition 2024)
- **Framework**: Actix-web
- **Database**: PostgreSQL with sqlx
- **Search**: Meilisearch
- **API Documentation**: OpenAPI 3.0 with utoipa
- **Testing**: rstest, insta snapshots, testcontainers

### Frontend (Webclient)
- **Framework**: Nuxt 4 with Vue 3
- **Language**: TypeScript
- **Styling**: Tailwind CSS with nightwind for dark mode
- **Map**: MapLibre GL
- **Package Manager**: pnpm
- **Linting/Formatting**: Biome

### Data Processing
- **Language**: Python
- **Key Libraries**: polars, pydantic, beautifulsoup4, pyyaml
- **Formatting**: ruff (via pre-commit)

## Development Workflow

### Setup
- The project uses Docker Compose for local development
- Run `docker compose -f compose.local.yml up --build` for a local dev environment
- The local build skips heavy geodata initialization (valhalla, nominatim, planetiler, martin)

### Formatting and Linting
- **Pre-commit hooks** are configured for automatic formatting
- Install with: `pre-commit install`
- Run manually: `pre-commit run --all-files`

#### Language-Specific Commands:
- Formatting is automatically handled by ruff via pre-commit.

### Testing
- **Server**: `cargo test`
- **Webclient**: Type checking with `npm run --prefix webclient type-check`
- **E2E tests**: Defined in `.github/workflows/e2e-tests.yml`

## Code Style and Preferences

### General
- Use modern, idiomatic code for each language
- Follow existing patterns in the codebase
- Maintain consistency with surrounding code
- Write descriptive commit messages

### Rust
- Use Rust 2024 edition features
- Prefer Result types over panics
- Use actix-web patterns for route handlers
- Follow Rust API guidelines
- Use sqlx compile-time verification for database queries
- Leverage workspace dependencies defined in root `Cargo.toml`

### TypeScript/Vue
- Use TypeScript strict mode
- Use Composition API
- Use auto-imports where configured
- Follow Vue 3 best practices
- Use Tailwind utility classes
- Dark mode is automatically provided via nightwind

### Python
- Use type annotations
- Follow PEP 8 (enforced by ruff)
- Use pydantic for data validation
- Use polars for data processing

## Important Files and Directories

### Configuration Files
- `openapi.yaml` - API specification (source of truth, kept in sync by CI from the server)
- `compose.yml`, `compose.local.yml` - Docker orchestration
- `Cargo.toml` (root) - Rust workspace configuration
- `pyproject.toml` - Python project configuration
- `.pre-commit-config.yaml` - Pre-commit hooks configuration

### Key Directories
- `/server/src` - Rust backend source code
- `/server/migrations` - Database migrations
- `/webclient/app` - Nuxt application code
- `/webclient/content` - Nuxt content (markdown pages)
- `/data/sources` - Data source definitions
- `/data/processors` - Data processing scripts
- `/data/output` - Generated data files (not in git)

## API Guidelines

- The API is documented in OpenAPI 3.0 format
- Interactive documentation available at `/api` endpoint
- All endpoints should be documented with utoipa macros
- Type regeneration: `npm run --prefix webclient type-refresh`
- API is still evolving - breaking changes are possible

## Dependencies

### Adding Dependencies
- **Rust**: Add to workspace dependencies in root `Cargo.toml` when possible
- **Node.js**: Use `pnpm add` in webclient directory
- **Python**: Add to `data/requirements.txt` or `server/test/requirements.txt`

### Updating Dependencies
- Renovate bot handles automatic dependency updates

## Testing Guidelines

- Write tests for new functionality
- Use snapshot testing (insta) for Rust when appropriate
- Use testcontainers for integration tests requiring databases
- Ensure tests pass before submitting PRs
- E2E tests run in CI but can be expensive locally

## Documentation

- Update README.md for user-facing changes
- Update component READMEs for component-specific changes
- Update OpenAPI spec for API changes
- Document breaking changes in PR descriptions
- Follow existing documentation style

## Performance Considerations

- **Rust compilation**: First builds are slow; incremental builds are faster
- **Local development**: Geodata services are intentionally skipped for faster startup

## Deployment

- CI/CD workflows are in `.github/workflows/` that push docker images. Prod auto-updates.
- Security scanning is configured in `.github/workflows/security.yaml`
- Automatic data updates via `.github/workflows/update-data.yml`

## Contributing

- Read `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md`
- Discuss major changes via issues first
- Small changes (typos, minor fixes) can skip issue creation
- Ensure pre-commit hooks pass
- Update documentation as needed
- Get one reviewer approval before merging

## Common Commands

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

## Notes for AI Assistants

- Always check existing patterns before implementing new features
- Consider the multi-language nature of the repository
- Test changes in the appropriate component
- Use docker compose for integration testing
- Check OpenAPI spec before making API changes
- Remember that the project serves students - usability matters
- Security is important - this is a public-facing service
