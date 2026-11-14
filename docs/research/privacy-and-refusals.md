# Privacy, content-policy claims and refusals

Reference evidence, checked 2026-11-14. No moderation probes or paid generations
were performed. The application requirements are in the foundations; this file
records what the provider actually documents and what remains unknown.

## Separate claims, not a single trust badge

Venice says its proxy does not retain normal inference prompts/responses.
Its `private` mode additionally asserts contract-enforced zero retention at the
processing layer. `anonymized` obscures the customer's identity from an upstream
provider but exposes the content to that provider; it does not itself establish
that provider's retention or training terms. Operational metadata can still be
processed, including request/account identifiers, timestamps, usage, billing and
network information. These are Venice's claims, not an independent audit.
[Venice privacy documentation](https://docs.venice.ai/overview/privacy)

| UI fact | Evidence | Honest interpretation |
| --- | --- | --- |
| Private | `model_spec.privacy: private` | Venice claims no prompt/output retention for this route; does not imply self-hosting or no moderation |
| Anonymous | `model_spec.privacy: anonymized` | Identity hidden from provider; upstream content retention not established by this field |
| Unknown privacy | Missing/unrecognized field | Preserve the actual value and do not infer protection |
| Uncensored claim | Dated model description or explicit trait | A named provider claim, not a promise never to refuse |
| No training / residency / retention period | Specific applicable route policy, if available | Unknown until supported; do not infer from a marketing label |
| Local retention | The app's autosave contract | Images and recipes remain on the owner's disk independently of provider retention |

The captured generation catalog has 11 private and 37 anonymized models. Private
IDs are `venice-sd35`, `grok-imagine-image-2-0`, `krea-2-turbo`,
`hunyuan-image-v3`, `grok-imagine-image`, `lustify-sdxl`, `lustify-v7`,
`lustify-v8`, `wai-Illustrious`, `z-image-turbo`, and `chroma`.
All other captured generation IDs are anonymized. The [model guide](model-guide.md)
shows the classification for every entry. [Public catalog](https://api.venice.ai/api/v1/models?type=image)

TEE and E2EE are currently documented for text models only. Do not show an image
encryption claim merely because a generic platform guide discusses enclaves.
Transport HTTPS is different from client-to-model end-to-end encryption.
[Privacy modes](https://docs.venice.ai/overview/privacy)

## Model-level uncensored evidence

| Generation ID | Captured evidence |
| --- | --- |
| `lustify-sdxl` | [Venice describes an uncensored adult-oriented photographic checkpoint](https://docs.venice.ai/models/lustify-sdxl) |
| `lustify-v7` | [Same positioning for v7](https://docs.venice.ai/models/lustify-v7); catalog trait `most_uncensored` |
| `lustify-v8` | [Same positioning for v8](https://docs.venice.ai/models/lustify-v8); catalog trait `most_uncensored` |
| `wai-Illustrious` | [Venice advertises anime including uncensored adult content](https://docs.venice.ai/models/anime) |
| `chroma` | [Venice describes uncensored photographic generation](https://docs.venice.ai/models/chroma) |
| `seedream-v4` | [Venice describes its V4.5 deployment as uncensored](https://docs.venice.ai/models/seedream-v4-5) |

There is no universal `uncensored: true/false` capability in the captured catalog.
For other models, this sweep did not establish an equivalent model-specific
uncensored claim. That is an unknown, not a categorical “censored” label.
Platform-wide promotional wording must not overwrite route-specific evidence.
An upstream vendor policy may help explain possible restrictions, but should
not be presented as proof of the exact reason Venice returned a refusal.

`safe_mode: false` requests unblurred adult output. It does not disable a model
provider's policy. Venice says the web app's mature-filter setting does not
control API image requests, and no Pro subscription is needed for the unblurred
API option. [Image generation guide](https://docs.venice.ai/guides/media/image-generation)

## Refusal and error evidence matrix

| Observation | Information to preserve and explain | What cannot be concluded |
| --- | --- | --- |
| 422 with explicit policy error | Provider message, code/reason if present, `retryable`, optional `suggested_prompt`/`recommended_model`, request ID | Exact prohibited category if unstated; whether a different prompt/model will succeed |
| 200 with omitted variants | Actual images, requested count, `content_policy_violation_count`, `x-venice-content-violation-count` | Which input words caused rejection; a missing image that never arrived |
| Returned blur/violation flags | `x-venice-is-blurred`, `x-venice-is-content-violation`, returned bytes and aggregate scope | Per-image attribution when only response-wide flags exist |
| Dark/blank-looking image without flags | Actual returned file and available diagnostics | Censorship based on pixel appearance alone |
| Non-policy 422 | Body and field-specific failure, such as image-size constraints | Policy refusal from HTTP status alone |
| Missing/invalid key or no funds | Distinguish auth challenge, invalid credentials, balance/key-spend limits from body and status | That every 402 means insufficient account balance |
| 400/403/413/429/5xx or unknown shape | Validation/access/size/retry-timing/provider diagnostics as supplied | That these are censorship, or that automatic retry is free |
| Interrupted body/connection | Captured partial evidence and unknown remote outcome | No generation occurred, no charge, or remote cancellation |
| Refund field/message present | Exact provider assertion, separately from independently verified billing | A refund rule for every image refusal from a video example |

The [native endpoint](https://docs.venice.ai/api-reference/endpoint/image/generate)
documents omitted variants, blacked-out flagged images and the policy-error shape.
The prose says `retryable` accompanies each policy rejection, while the schema
requires only `error`; tolerate absence. `retryable: true` describes possible
nondeterministic moderation, not a guarantee of success. All rejected variants
can return 422; surviving variants can arrive in a 200.
[Generation guide](https://docs.venice.ai/guides/media/image-generation)

The [general error reference](https://docs.venice.ai/api-reference/error-codes)
shows strings and nested error objects, validation arrays, optional codes and
endpoint-dependent fields. Missing credentials can produce a 402 x402 challenge;
invalid supplied credentials normally produce 401. Its refund example is for
video, so image refunds remain unknown unless the actual image response supplies
evidence. This app uses API-key authentication; x402 wallet payments are not in scope.

## Explanation boundary

Proposed example summary: “Venice returned 2 of 4 images; it reports that 2 were
omitted by the provider's content policy. No more specific reason was supplied.”
The details view retains the exact available explanation. This is an authored UI
example, not a captured incident. Avoid both collapsing everything to “generation
failed” and fabricating an elaborate explanation the provider never gave.

Full explanation means all available useful evidence, subject to explicit bounded
capture and credential redaction. It cannot reveal private classifier reasoning
that was never returned. Save diagnostics privately, report truncation, escape
provider text, and keep speculative interpretations labeled. No app-side refusal
classifier or LLM explanation service is needed to meet this requirement.
