# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

This repo is **multi-context**: a single monorepo with three distinct stacks, each with its own domain vocabulary.

## Before exploring, read these

- **`CONTEXT-MAP.md`** at the repo root - points at one `CONTEXT.md` per context. Read each one relevant to the topic.
- The per-context **`CONTEXT.md`** for the stack you're working in (see layout below).
- **`docs/adr/`** at the repo root for system-wide decisions, plus **`<stack>/docs/adr/`** for stack-specific decisions touching the area you're about to work in.

If any of these files don't exist yet, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The producer skill (`/grill-with-docs`) creates them lazily when terms or decisions actually get resolved.

## File structure

```
/
├── CONTEXT-MAP.md
├── docs/adr/                ← system-wide / cross-stack decisions
├── server/                  ← Rust backend
│   ├── CONTEXT.md
│   └── docs/adr/
├── webclient/               ← Vue / TypeScript frontend
│   ├── CONTEXT.md
│   └── docs/adr/
├── data/                    ← Python data-collection & areatree pipeline
│   ├── CONTEXT.md
│   └── docs/adr/
├── map/                     ← map-tile build pipeline
└── resources/               ← static assets, deploy config
```

`map/` and `resources/` are small enough that they share system-wide decisions in the root `docs/adr/` and don't carry their own `CONTEXT.md`.

## Pick the right context

- Backend / API / database / search-index work → `server/CONTEXT.md` (+ `server/docs/adr/`)
- Frontend / UI / Vue component work → `webclient/CONTEXT.md` (+ `webclient/docs/adr/`)
- Data scraping, areatree, building data, coordinate fixes → `data/CONTEXT.md` (+ `data/docs/adr/`)
- Cross-stack architecture (deploy, contracts between stacks, openapi.yaml) → root `docs/adr/`

When in doubt, consult `CONTEXT-MAP.md`.

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in the relevant `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids - and don't mix vocabulary across stacks (e.g. backend "entry" vs. frontend "detail view" are not always interchangeable).

If the concept you need isn't in the glossary yet, that's a signal - either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/grill-with-docs`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts `server/docs/adr/0007-search-index-layout.md` - but worth reopening because…_
