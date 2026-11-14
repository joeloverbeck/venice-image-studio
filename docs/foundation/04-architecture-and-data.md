# Architecture and data

## Small local application

One Rust process serves the local API and built browser assets. Rust owns Venice
HTTP, credentials, validation, operation state, persistence and authoritative
catalog interpretation. TypeScript owns presentation, input interaction and
view state; it does not call Venice or duplicate business rules. Choose ordinary
maintained Rust libraries during implementation. No framework, database or
general provider abstraction is justified merely by this foundation.

Bind to loopback. Validate Host/Origin and protect state-changing requests from
cross-site browser calls; loopback alone is not sufficient. Serve images by known
library IDs, never arbitrary client-supplied filesystem paths. Bound HTTP bodies
and image decoding, validate actual formats, and render external descriptions and
errors as text. Do not embed third-party scripts, trackers or remote image previews.

Read `VENICE_API_KEY` only in Rust from the ignored repo-local `.env` or explicit
process environment, with process environment taking precedence. Never overwrite
an existing `.env`. Supply only an empty `.env.example`; recommend owner-only
permissions. Credentials must never reach browser bundles, responses, recipes,
logs, fixtures or Git. Do not borrow another repository's key. Missing or invalid
credentials should produce an actionable setup message without exposing them.

## External library and durability

Default library: `~/Pictures/Venice Image Studio`, resolved against the user's
home directory. `VENICE_IMAGE_STUDIO_DATA_DIR` allows another absolute location.
Keep images, recipes, operation history, drafts, thumbnails and runtime caches
outside the repository. Resolve symlinks and reject a library path inside any
Git working tree, including worktrees. A clean clone and deleting the checkout
must leave the library untouched. Start with files and JSON sidecars; any index
is rebuildable and added only for demonstrated library needs.

Use collision-resistant operation/image identifiers and filenames without prompt
text. Preserve the exact returned image bytes; do not re-encode the original.
Save via temporary files and atomic finalization, with explicit recovery for
crashes between image and sidecar writes. Check writability and reasonable free
space before dispatch; a later disk failure must leave a visible recoverable
state. Never report “saved” before durable completion. No retention cleanup runs
without the owner's explicit choice.

Each operation's private recipe preserves the minimum information needed to
understand and repeat the attempt:

- Recipe format version, app version, local identity, creation/dispatch/completion
  times and parent identity when reusing an earlier recipe.
- Exact original prompt, negative prompt if used, exact submitted generation
  settings, explicit defaults and omitted-versus-sent distinctions. Never store
  authorization headers or a credential-bearing raw request.
- Model ID/name, relevant dated constraint/capability/privacy snapshot, source
  references, requested seed and actual seed only if returned.
- Requested count/size/format and per-result actual dimensions, detected type,
  bytes, relative library path and SHA-256; original/thumbnail distinction.
- Provider request/response identity and timings where supplied, outcome,
  moderation/blur flags, omission counts, useful redacted refusal/error details,
  suggestions, truncation and incomplete-body evidence.
- Applicable price estimate and source date, separately from any verified charge
  or refund. An absent amount stays unknown.

Do not duplicate base64 payloads in recipes or keep raw HTTP headers by default.
Use sidecars for prompts; `embed_exif_metadata` starts false. Original files may
already contain provider metadata, so an original download is not promised to be
metadata-free. Exporting a recipe is an explicit action separate from downloading
the image. All returned images, including flagged images, are retained with their
status; refused attempts retain metadata without fabricating an image.

## Privacy and shared-code boundary

Autosave deliberately retains content locally even where a provider advertises
no retention. Explain that distinction and the library location. Use owner-only
permissions where supported; do not promise encryption at rest. Ordinary logs
contain operational identifiers/status, not prompts, credentials, images or full
refusal bodies. Private diagnostic details belong to the local operation; export
must be deliberate and redacted. No analytics or automatic uploads.

Do not depend on or rename `openrouter-transport`. At reviewed commit
`2e27b7c5b9894c0e6a20f02469605dedc4b5d8b8`, its host and generation route are
OpenRouter/chat-specific. Adopt relevant lessons (single attempt, no redirects,
bounded evidence), using a small Venice client in this repository. Existing
consumers need no changes. A shared transport extraction requires a real new
consumer contract and separate scope; see [source review](../research/venice-api.md#shared-transport-review).
