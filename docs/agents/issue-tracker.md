# Issue tracker: GitHub

Issues and specs live in `joeloverbeck/venice-image-studio` on GitHub.
GitHub Issues and pull requests are the sole active work-control plane.
Local documents support that work; they do not form a second ticket queue.

## Authority

Follow `docs/foundation/05-delivery-and-acceptance.md` for work activation and
acceptance. A specification, dependency state or readiness label does not
activate implementation. Record the owner's selected outcome, bounds,
verification and publication scope in the relevant issue.

## Conventions

Run `gh` inside this checkout, or pass
`--repo joeloverbeck/venice-image-studio`. Before the first write in a session,
verify the target with `gh repo view --json nameWithOwner`.

- Create: `gh issue create --title "..." --body-file /path/to/body.md`.
- Read: `gh issue view <number> --json number,title,body,labels,comments`.
- List: `gh issue list --state open --limit 100 --json number,title,body,labels`.
  Use appropriate filters and pagination when complete enumeration is needed.
- Update: `gh issue edit <number> --body-file /path/to/body.md`.
- Comment: `gh issue comment <number> --body-file /path/to/comment.md`.
- Label: `gh issue edit <number> --add-label "..."` or `--remove-label "..."`.
- Close: post the explanation as a comment, then `gh issue close <number>`.
  Post comments separately so they are retained even if the issue is already closed.

Use body files for multiline text. File issues only in the owner's selected repo.
When a skill says "publish to the issue tracker", create a GitHub issue within
its authorized scope. "Fetch the relevant ticket" means read the issue above.

## Parents and blockers

Use native sub-issues: `gh issue create --parent <parent> ...` or
`gh issue edit <parent> --add-sub-issue <child>` (gh 2.94+).
Use native dependencies: `gh issue edit <child> --add-blocked-by <blocker>`.
If unavailable, use `Part of #<parent>` and `Blocked by: #<number>` lines in the
child body, with a task-list link from its parent.

Count open blockers with
`gh api repos/joeloverbeck/venice-image-studio/issues/<number> --jq .issue_dependencies_summary.blocked_by`.
Zero open blockers removes that dependency constraint; accepted scope and owner
activation still govern whether work may start.

## Pull requests as a triage surface

**PRs as a request surface: no.**

GitHub shares issue and PR numbers. Resolve a bare number with `gh pr view <number>`;
if GitHub reports that it is not a PR, use `gh issue view <number>`. Authentication
or network errors are not evidence of object type.

## Wayfinding operations

When the owner explicitly selects wayfinding, keep its map in one GitHub issue
with `wayfinder:map` and its tickets as native sub-issues. Ticket labels are
`wayfinder:research`, `wayfinder:prototype`, `wayfinder:grilling` or `wayfinder:task`.
Use the parent/blocker conventions above and map order to find unassigned open
children without open blockers. Claim only activated work with
`gh issue edit <number> --add-assignee @me`.
Record the result in the child, close it when its accepted scope is satisfied,
and add a concise result/link to the map's Decisions-so-far. This setup creates
no map, tickets or wayfinding labels.
