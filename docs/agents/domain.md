# Domain docs

This repository uses a single-context layout: `GLOSSARY.md` at the root and
architecture decision records in `docs/adr/`.

## Before domain exploration

Read the root `GLOSSARY.md` and ADRs relevant to the area being explored, when
present. If they are absent, proceed silently; do not create placeholders.
The domain-modeling skill can record terms and decisions when they are resolved.
The repository entry points in `CLAUDE.md` remain the accepted product context.

## Vocabulary and decisions

Use glossary terms consistently in issues, proposals, code and tests. When a
concept is missing, first check whether an existing term fits; identify a real
gap when it does not.

If a proposal contradicts an ADR, name the ADR and explain the proposed change
and rationale. Surface the conflict for resolution instead of silently replacing
the accepted decision. Domain documents record vocabulary and decisions;
work activation and status belong to GitHub Issues and pull requests.
