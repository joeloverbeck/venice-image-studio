# Venice Image Studio

A planned local image studio for Venice AI: Rust application logic, a thin
TypeScript browser interface, and a permanent image library outside the checkout.

**Status: researched foundations only. There is no runnable application yet.**
No paid generations or model-quality benchmarks were performed during founding.

The first usable version will support long prompts within each model's limits,
informed model selection, appropriate sizing and seed controls, image inspection,
downloads, and automatic saving of every result with its recipe. Safe mode starts
off; optional prompt enhancement stays off. Refusals, partial results, privacy
claims and uncensored-model claims must be explained clearly.

## Start here

- [Purpose and scope](docs/foundation/01-purpose-and-scope.md)
- [User experience](docs/foundation/02-user-experience.md)
- [Venice API contract](docs/foundation/03-venice-contract.md)
- [Architecture and saved data](docs/foundation/04-architecture-and-data.md)
- [Delivery and acceptance](docs/foundation/05-delivery-and-acceptance.md)
- [All 48 researched generation models](docs/research/model-guide.md), including
  prompting guidance, prices, limits, privacy and content-policy claims
- [API and transport research](docs/research/venice-api.md)
- [Privacy and refusal evidence](docs/research/privacy-and-refusals.md)

The research is dated **2026-11-14**. Runtime model availability, prices and
capabilities must be refreshed from Venice. Some models still cap prompts at
1,500 characters; advertised larger limits are not proof of successful requests.

The intended private configuration is a repo-local `.env` containing
`VENICE_API_KEY`; [.env.example](.env.example) contains no credential. The intended
default library is `~/Pictures/Venice Image Studio`, overridable with
`VENICE_IMAGE_STUDIO_DATA_DIR`. Neither credentials nor generated work belongs
in Git. These are future application contracts, not working setup instructions.

[GitHub Issues and pull requests](https://github.com/joeloverbeck/venice-image-studio)
are the work-control plane. The [repository constitution](docs/principles/principles-for-repos.md)
governs all specifications and tickets. `CLAUDE.md` contains the agent bootstrap;
`AGENTS.md` is its relative symlink.

MIT licensed. Independent project; not affiliated with Venice AI or model vendors.
The application license does not determine provider terms or generated-image rights.

## Mock-ups

Use [VectorCraft](https://github.com/storytold/vectorcraft) through its MCP server
for UI mock-ups. The development checkout is the sibling `../vectorcraft`.
This is a design tool; the Venice application does not depend on it at runtime.

The project-scoped Codex connection is in `.codex/config.toml`. It runs the
installed CLI in headless mode, with file access restricted to
`${XDG_DATA_HOME:-$HOME/.local/share}/venice-image-studio/mockups`. Keep editable
`.vectorcraft` documents and their PNG/SVG exports there, outside Git. Retain the
editable original when sharing a preview. Mock-ups propose UI choices; they do
not establish working Venice behavior or owner acceptance.

Setup from this checkout, with Rust and VectorCraft's source-build prerequisites:

```sh
git -C ../vectorcraft status --short --branch
# Continue only from a clean main checkout; preserve ignored artwork.
git -C ../vectorcraft pull --ff-only origin main
cargo build --manifest-path ../vectorcraft/Cargo.toml --locked -p vectorcraft -p vectorcraft-cli
cargo install --path ../vectorcraft/apps/vectorcraft-cli --locked
mkdir -p "${XDG_DATA_HOME:-$HOME/.local/share}/venice-image-studio/mockups"
codex mcp get vectorcraft
```

Restart the Codex session after installation or configuration changes to load the
connection. It requires a trusted project and the CLI at
`${CARGO_HOME:-$HOME/.cargo}/bin/vectorcraft-cli`; a missing binary or artifact
folder is a setup error. Rebuild and reinstall after updating VectorCraft, and
record the source commit when checking a build: the version number alone does
not identify its source. The desktop executable is
`../vectorcraft/target/debug/vectorcraft`. MCP does not require a running window.

Verified on 2026-11-14 against VectorCraft `main` commit
`edb228ac33409473af2102e0825081cb70c50283` (version 0.24.1): desktop and CLI
builds succeeded, the installed MCP exposed 33 tools, and a document containing
editable shapes and text saved and exported to PNG/SVG. Reopening the native
document in a fresh MCP process produced an identical PNG. The
`setup-check-edb228ac3.*` files in the mock-up folder are setup evidence, not a
proposed application design. This verifies the tool connection and file workflow;
desktop interaction and Venice application usability were not tested.
