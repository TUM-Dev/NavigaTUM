# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues on the upstream **`TUM-Dev/NavigaTUM`** repository. Use the `gh` CLI for all operations.

## Repo targeting

The local `origin` remote points at a personal fork (`CommanderStorm/NavigaTUM`). All issue and PR operations should target the upstream instead — pass `--repo TUM-Dev/NavigaTUM` explicitly to `gh` so commands don't default to the fork.

## Conventions

- **Create an issue**: `gh issue create --repo TUM-Dev/NavigaTUM --title "..." --body "..."`. Use a heredoc for multi-line bodies.
- **Read an issue**: `gh issue view <number> --repo TUM-Dev/NavigaTUM --comments`.
- **List issues**: `gh issue list --repo TUM-Dev/NavigaTUM --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'` with appropriate `--label` and `--state` filters.
- **Comment on an issue**: `gh issue comment <number> --repo TUM-Dev/NavigaTUM --body "..."`
- **Apply / remove labels**: `gh issue edit <number> --repo TUM-Dev/NavigaTUM --add-label "..."` / `--remove-label "..."`
- **Close**: `gh issue close <number> --repo TUM-Dev/NavigaTUM --comment "..."`. To close as "not planned" (the `wontfix` role), pass `--reason "not planned"`.

## Discussions, not Issues, for webform & data

Webform-originated reports and data-correction requests are tracked in **GitHub Discussions** on `TUM-Dev/NavigaTUM`, not Issues. When triaging or surveying:

- Don't open new Issues for webform/data items — check Discussions first and route there.
- Use `gh api repos/TUM-Dev/NavigaTUM/discussions` (or the equivalent GraphQL query) to list/search.
- Look for broad-class duplicates when surveying — many incoming items collapse into one umbrella thread.

## When a skill says "publish to the issue tracker"

Create a GitHub issue on `TUM-Dev/NavigaTUM` (or a Discussion, if it's a webform/data item).

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --repo TUM-Dev/NavigaTUM --comments`.
