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
