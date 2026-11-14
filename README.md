# Venice Image Studio

A local image studio for Venice AI: Rust application logic, a thin
TypeScript browser interface, and a permanent image library outside the checkout.

**Status: all-model browsing and generation-controls candidate.**
The Writing desk supports the current catalog, dated model guidance, validated
model-specific controls, and saving/reopening original images and recipes.
The first Chroma provider checkpoint is recorded in [issue #2](https://github.com/joeloverbeck/venice-image-studio/issues/2).
[Issue #3](https://github.com/joeloverbeck/venice-image-studio/issues/3) owns the
current model-choice increment; its Grok checkpoint and owner evaluation remain
pending. Offline checks establish client behavior, not model quality, effective
seeds, provider guarantees or owner usability. [Issue #1](https://github.com/joeloverbeck/venice-image-studio/issues/1)
owns the complete delivery and acceptance boundary.

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
in Git. The backend implements these locations; see the setup below.

[GitHub Issues and pull requests](https://github.com/joeloverbeck/venice-image-studio)
are the work-control plane. The [repository constitution](docs/principles/principles-for-repos.md)
governs all specifications and tickets. `CLAUDE.md` contains the agent bootstrap;
`AGENTS.md` is its relative symlink.

MIT licensed. Independent project; not affiliated with Venice AI or model vendors.
The application license does not determine provider terms or generated-image rights.

## Run locally

Requires Ubuntu, Rust/Cargo (edition 2024; checked with 1.97.1), Node.js 24 and npm.
From a clean checkout, set `VENICE_API_KEY` in the process environment or create
an owner-readable repo-local `.env` from `.env.example` (`chmod 600 .env`). Existing
process variables take precedence. The app never overwrites this file. A missing
key still permits browsing the local app and gives setup guidance; Generate is
disabled. An invalid key produces a redacted access error after Venice responds.

```sh
./scripts/start
```

This installs pinned frontend dependencies, builds the browser assets and Rust
binary, then runs one process. Open **http://127.0.0.1:4318**. Subsequent launches
can run `./target/debug/venice-image-studio` directly; rerun the script after code
changes. `--port 4319` selects another loopback port. Use the printed IPv4 address.

Browse and compare every current generation model by name, ID and dated practical
guidance, with separate filters for uncensored positioning, Venice privacy,
upstream retention and additional protection. Inspect source links, exact limits,
sizing choices and the price table in each model detail. These are attributed
claims and editorial suggestions, not a benchmark. Unknown and removed models
remain visible; incomplete contracts prevent dispatch without substituting a model.

Model changes retain the prompt and settings. Resolve visible incompatibilities
or explicitly choose **Use this model’s documented defaults** before generating.
Pixel routes validate bounds/divisors; ratio routes send only advertised ratios,
tiers and qualities. Requested size and actual returned dimensions stay separate:
the first Chroma checkpoint requested 1024×1024 and returned 1536×1536.

One explicit Generate submits one attempt with one image by default. Advanced
controls allow 1–4 images; the count reaches the total price estimate, request,
recipe and reuse. Safe mode starts off and remembers later choices; enhancement and embedded metadata stay disabled. Advanced controls appear
with documented support, uncertainty and sources. Seed omission has Venice’s
random/default-zero discrepancy; an explicit seed never promises identical output.
Web search and optional optimization begin off. The latter uses the provider’s
negatively named disable field. Image-search surcharge information conflicts, so
search-enabled dispatch remains unavailable with base price and unknown extra cost
shown. No app setting promises to disable internal model rewriting.

Catalog and styles refresh at startup and through **Refresh catalog**, with dated
external caches and visible failures. Applicable quality/resolution matrices govern
Rust preflight estimates; the displayed estimate is bound to the exact draft and
catalog revision before dispatch. Estimates are neither actual charges nor spending
caps. There are no automatic retries or fallback models. Catalog/style refreshes
are read-only. Detailed source applicability is recorded in the
[control evidence](docs/research/control-contract.md).

The default library is `~/Pictures/Venice Image Studio`. To override it, set
`VENICE_IMAGE_STUDIO_DATA_DIR` to an absolute external directory before launch.
The directory must be outside every Git working tree, including linked worktrees;
symbolic links in its path are rejected, so use its actual location. The app sets
library permissions to 0700 and new private files to 0600. It requires 256 MiB free
before dispatch and holds a library lock against concurrent Studio processes.
Checkout updates or removal do not remove this library. Retention is local,
without encryption or automatic cleanup.

Results keep requested, received, invalid and provider-reported omitted counts
separate. Response-wide blur and content-violation flags remain visible alongside
saved originals, including when reopening older recipes. These flags do not identify
individual images or explain the policy reason. Structured policy codes and the
native 422 retry-advice field can identify a refusal; arbitrary error text, a 422
alone or black pixels cannot. Unknown error shapes remain inspectable within
bounded, redacted details. Provider billing/refund assertions are unverified;
missing amounts remain unknown. Suggestions and retry advice never change or
resubmit a draft.

Drafts autosave; wait for **Draft saved locally** before closing. Reload reconnects
to an in-flight attempt. A process stop after dispatch becomes **indeterminate**
with unknown billing, without replay. A local saving failure keeps received bytes
available in the running app: download them or use **Retry local save**, which
never generates again. Restart can recover fully written originals from the
pending recipe and shows which originals are unavailable. Missing or damaged
bytes after process loss cannot be recovered by a save retry. An interrupted native
JSON response retains complete originals already received; an incomplete image
cannot be reconstructed. A new Generate is a separate paid attempt, and a failed
or interrupted attempt may already have been charged. Keep Studio
open during a save failure. **Use recipe as new draft** makes an editable copy
without changing the previous recipe or making a paid request. Original downloads
preserve provider bytes, which may already contain provider metadata; the recipe
and diagnostics remain separate in the app and external library.

Local resource policy: request bodies 256 KiB, catalog 4 MiB, provider response
192 MiB, each original 96 MiB, decoded dimensions at most 8192×8192 with a 256 MiB
decoder allocation limit, and generation timeout five minutes. Diagnostic captures
are bounded and truncation is recorded. These are application safeguards, not
provider output guarantees. A synthetic 4096×4096 RGBA PNG over 64 MiB exercises
original retention, download and restart; other maximum-size combinations remain
bounded rather than universally verified.

## Check without credentials or paid calls

```sh
npm ci --ignore-scripts
npm run build
cargo fmt --check
cargo clippy --locked --all-features -- -D warnings
cargo build --locked --features test-support
python3 tests/workflow.py
```

The checks also require a C compiler for the synthetic filesystem-space fault.
The high-seam checks start the actual Rust server with a controlled loopback HTTP
provider and a temporary external library under `/var/tmp`. They cover exact long
Unicode prompts/settings, original/download/recipe identity, restart/reuse,
duplicate dispatch protection, interrupted requests, a crash between original and recipe finalization, local
save retry, missing/invalid credentials, storage/resource limits, all 48 researched base request paths, model-switch conflicts, stale
catalogs/styles, documented controls, Unicode limits, price matrices, variants and response-wide flags, structured/unknown failures,
partial/corrupt/interrupted images, low-space/permission failures, redirects, secret redaction and
Host/Origin/token/path boundaries. Fixtures are synthetic and not quality evidence.
The `test-support` feature enables `--fixture-provider` for loopback only, ignores
`.env` and process credentials, and sends a fixed synthetic credential. Ordinary
builds reject that option. Rebuild with `cargo build --locked` before ordinary use.
Browser checks of the same running path are recorded in the active issue.

Runtime source basis: public catalog refreshed 2026-11-14,
SHA-256 `faaf625846a2639e12ad5a9f1c251ea7993dea942adf3a8c29739c87d01c226e`, and
[Venice native generation contract](https://docs.venice.ai/api-reference/endpoint/image/generate),
OpenAPI SHA-256 `c100cced1ce996fe1f7844bb5e27115ff89cc02c74132a045c8e00c0347d4a59`.
These matched the dated research. Runtime availability and prices are refreshed;
source agreement does not itself establish live generation or owner acceptance.

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
