# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the actual mechanics used on `TUM-Dev/NavigaTUM`.

| Canonical role    | Mechanic on `TUM-Dev/NavigaTUM`                                  |
| ----------------- | ---------------------------------------------------------------- |
| `needs-triage`    | Apply label `needs-triage`                                       |
| `needs-info`      | Apply label `needs-info`                                         |
| `ready-for-agent` | Apply label `ready-for-agent`                                    |
| `ready-for-human` | Apply label `ready-for-human`                                    |
| `epic`            | Apply label `epic`. See "Epics" below.                           |
| `wontfix`         | Close the issue as "not planned" - no label. See command below.  |

All labels already exist on `TUM-Dev/NavigaTUM`; no creation step is required.

## Epics

An `epic` is a **meta-issue** that tracks a multi-slice initiative and describes the motivation and problem behind it, not the individual slices.
The individual `ready-for-*` should ideally have one such GitHub sub-issue as a parent.
The epic itself is not directly grabbable - so it carries `epic` *instead of* a `ready-for-*` role, never both.
Its sub-issues carry the `ready-for-*` roles.

When promoting an issue to an epic, swap the role rather than stacking it:

```
gh issue edit <number> --repo TUM-Dev/NavigaTUM --add-label epic --remove-label ready-for-agent
```

## Closing as "not planned" (the `wontfix` role)

```
gh issue close <number> --repo TUM-Dev/NavigaTUM --reason "not planned" --comment "<rationale>"
```

Always include a comment explaining why - drive-by closes without rationale are unhelpful to reporters.

## Notes on label hygiene

- The older label `information requested` exists on the repo but is superseded by `needs-info` - don't add it to new issues.

When a skill mentions a role (e.g. "apply the AFK-ready triage label"), look up the mechanic in this table and execute it against the upstream repo.
