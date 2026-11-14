# Venice image API research

Research date: 2026-11-14. Public documentation and unauthenticated catalog reads;
no paid generation or account inspection. This is reference evidence for the
foundation decisions, not a claim that the application exists or works.

## Decision supported

A Rust service can call Venice's native image endpoint, decode the returned image,
save it outside the source tree, and serve it to a local browser. The response
contract supplies the image bytes; a provider-hosted download button is not a
prerequisite. The owner has selected generation and a local image library first,
on Ubuntu for personal use. Editing and upscaling are deferred.

The required path is `POST https://api.venice.ai/api/v1/image/generate`. Its
OpenAI-compatible sibling has a smaller control surface and is unsuitable as the
primary route for this product. [Generation guide][generation]

## Discovery and model guidance

The captured `GET /models?type=image` response contains 48 image-generation
models. Each has a model ID, name, description, availability, privacy label,
constraints and prices. Some also provide source links, selection traits and
explicit capability flags. The companion edit catalog contains 33 entries;
the public style list contains 76 presets. These are dated inventory counts.
[Catalog][catalog] [Styles][styles]

Model browsing should combine live provider descriptions with practical filters:
prompt capacity, sizing method, price, declared privacy, availability and supported
features. Quality and speed labels must be attributed to Venice rather than
presented as application-tested rankings. Store exact IDs; a friendly name may
differ substantially (`seedream-v4` is currently named Seedream V4.5).

Known low-cost options with more than 1,500 advertised prompt characters include
Z-Image Turbo and Chroma, each currently listing 7,500 and $0.01 base generation.
This is catalog evidence, not a personal quality recommendation or tested prompt
acceptance. Some image models still have a 1,500-character limit on the native
endpoint. Runtime discovery matters more than a permanent bundled list.

## Request contract and material conflicts

| Concern | Evidence | Consequence for the app |
| --- | --- | --- |
| Prompt capacity | Model limits range from 1,500 to 32,768; generic OpenAPI schema still says `maxLength: 7500` | Show model limit and source; retain the discrepancy. Verify long requests in later authorized testing. Never truncate silently. |
| Counting | Documentation specifies characters, without definitive Unicode counting semantics | Character count is primary; no invented exact token count. Test emoji, combined accents and non-Latin input at the boundary. |
| Pixel sizes | Pixel models expose a divisor; generic width/height schema caps each at 1,280 | Validate both divisibility and bounds; give aspect presets as a convenience. |
| Ratio/tier sizes | Other models expose aspect ratios and sometimes resolution tiers | Use those fields. Several Qwen models explicitly reject width/height. Show returned pixel dimensions separately. |
| Steps/CFG | Shared fields exist; example-model docs say steps may be ignored | A field's presence in generic schema or catalog does not establish that it has an effect. Explain support and omit irrelevant parameters. |
| Output | JSON base64 array or PNG/JPEG/WebP binary | JSON supports 1–4 variants and metadata. Save original bytes; thumbnails are derived. |
| Partial output | Successful status can contain fewer variants and a content-policy omission count | Keep successful images, report omissions and never fill missing variants by automatic paid calls. |
| Safety | `safe_mode` controls adult-output blur; provider policy still applies | Expose the choice and returned blur/refusal status accurately. |
| Prompt enhancement | Optional rewrite, added cost/latency, final text header; may truncate rewritten prompt | Explicitly off for this product. Any unexpectedly returned revised text must remain separate from the original. |
| Transparency | Requires an advertised capability plus PNG/WebP | Do not offer effective alpha support for JPEG or an unsupported model. |
| Style references | Capability, count and strength support vary | A later reference workflow must check model limits; not a universal upload field. |

The schema/catalog prompt discrepancy is unresolved. The native route removes
the application's blanket chat limit only for models/routes that accept longer
input; it cannot promise unlimited prompts or override a model's actual limit.
[Native endpoint][native] [Generation guide][generation]

## Spending, failures and lifecycle

The model catalog contains flat, resolution-tier and resolution-by-quality
prices. Use the quality matrix when applicable; multiply image costs by requested
variants and account for selected extras. Estimates are not actual invoices or
server-enforced caps. Provider docs currently price enhancement at $0.04 per
applied rewrite, at most once across a request's variants, with up to about
30 seconds additional latency. Web search and reference images can incur their
own charges. [Model pricing][models] [Enhancement][enhancement]

Generation is synchronous. No image-generation retrieval, idempotency, or remote
cancellation guarantee was found in the consulted API contract. Preserve a local
operation identity before dispatch, distinguish an ambiguous interruption from
a known rejection, and do not automatically resubmit paid work. A page reload
must reconnect to the local operation, not repeat its POST. Saving already
received bytes again must not regenerate an image.

Rate limits vary by model and key. Read current metadata and response headers;
do not hardcode the advertised request rate of another account. Present retry
timing without silently reissuing a potentially charged image request. Errors
include invalid settings, authentication/access, insufficient funds, unavailable
models, policy rejection, throttling and provider failure. Status alone is not sufficient: 422 can mean a non-policy validation failure,
and 402 can be an authentication challenge. Inspect the body and documented
code as well; preserve unknown forms.
[Rate limits][limits] [Errors][errors]

## Privacy and storage

The catalog labels 11 image models `private` and 37 `anonymized` in this snapshot.
Venice's privacy documentation distinguishes zero-retention processing from
identity-obscured third-party processing; anonymization does not hide a prompt
from the upstream model provider. These are provider assertions, not guarantees
independently audited by this application. Image models have no documented E2EE
flow. [Privacy][privacy]

The local library needs its own private custody: images, prompts, recipes and
history outside Git; `.env` kept ignored in the repo and read only by Rust; no
prompt-bearing application logs or analytics. Public documentation and synthetic
test data do not require private library access. The app must preserve the
library independently of source-code deletion, updates and clean clones.

## Shared transport review

Inspected `joeloverbeck/openrouter-transport` at remote `main`
`2e27b7c5b9894c0e6a20f02469605dedc4b5d8b8`, including its complete Rust source.
The production host is fixed to OpenRouter and its generation request is fixed
to `chat/completions`. The only configurable alternative is explicit HTTP
loopback for fixtures. It cannot currently send Venice image requests.
[Source][transport]

Its relevant design lessons are one HTTP attempt, no redirects, bounded captures,
separate HTTP and body-read failures, and caller-owned decoding. Those lessons do
not justify adopting an incompatible dependency. `character-relationship` already
has a separate Venice client; its OpenRouter transport and
`narrative-explorer` both pin the above shared commit.

Recommendation: retain the existing library and name. Build only the local
Venice-specific transport needed by this app. No cross-repository migration is
warranted. A neutral shared transport would be a new contract, not a cosmetic
rename, and would need real consumer pressure and separate evidence.

## Deferred capabilities

Venice also supports single/multi-image edits, background removal and upscaling.
Their different response and input contracts are researched but not first-version
requirements. The old generation `inpaint` parameter is deprecated; it must not
be used to implement a future editor. Upscaling returns binary, preserves no
application-owned history by itself, and has size/scale constraints. Future work
must preserve originals and identify derived images. [Editing][editing]
[Upscaling][upscaling]

## Evidence still needed

Before claiming the app usable: real long-prompt generation on at least one
selected model; actual controls and returned dimensions; original-byte saving and
browser download; history after restart; a different model/sizing path; and owner
inspection of ease of use. Before promising prompts above 7,500 characters or
exact seed reproducibility, resolve those claims with suitable authorized
evidence. Offline fixtures can test handling; they cannot prove provider behavior
or experienced image quality.

[catalog]: https://api.venice.ai/api/v1/models?type=image
[styles]: https://docs.venice.ai/api-reference/endpoint/image/styles
[native]: https://docs.venice.ai/api-reference/endpoint/image/generate
[generation]: https://docs.venice.ai/guides/media/image-generation
[models]: https://docs.venice.ai/models/image
[enhancement]: https://docs.venice.ai/guides/media/prompt-enhancement
[limits]: https://docs.venice.ai/api-reference/rate-limiting
[errors]: https://docs.venice.ai/api-reference/error-codes
[privacy]: https://docs.venice.ai/overview/privacy
[transport]: https://github.com/joeloverbeck/openrouter-transport/blob/2e27b7c5b9894c0e6a20f02469605dedc4b5d8b8/src/lib.rs
[editing]: https://docs.venice.ai/guides/media/image-editing
[upscaling]: https://docs.venice.ai/guides/media/image-upscaling

## Additional current findings

The owner requires full available refusal details and explicit privacy/content-policy
claims for each model. See [privacy and refusal evidence](privacy-and-refusals.md)
for the error-shape matrix, source-attributed uncensored claims and retention
limitations. See the [complete model guide](model-guide.md) for all 48 models
and prompting research. No live refusal, seed or billing behavior was exercised.

The shared-code consumer inspection also read character-relationship at
`454dc8e6b3288f8f53e58b2ca44b4cd890420152` and narrative-explorer at
`c56a59a0e9f34cfac2ba7eda227da87b9c4b8ff5`. Their dependency manifests/locks
and provider boundaries support the no-rename decision; neither repository
was modified. This is source evidence, not a test of this future app.

Useful control details from the native schema: `cfg_scale` is greater than zero
and at most 20; generic negative prompts cap at 7,500; style names come from
`/image/styles`; `lora_strength` adjusts existing model support, not arbitrary
uploads; watermark hiding may be ignored. Support must be assessed per model.

Source capture also included the [OpenAPI schema](https://docs.venice.ai/swagger.yaml)
(SHA-256 `c100cced1ce996fe1f7844bb5e27115ff89cc02c74132a045c8e00c0347d4a59`)
and the [documentation index](https://docs.venice.ai/llms.txt). Public docs were
read rather than republished wholesale. Pricing is dated reference information,
not a quote to the owner or evidence of account access.
