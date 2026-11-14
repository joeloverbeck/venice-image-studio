# Venice API contract

Evidence and links: [API research](../research/venice-api.md) and
[refusal/privacy research](../research/privacy-and-refusals.md), checked 2026-11-14.
Provider contracts change; stored research is not runtime authority.

## Discovery and requests

Use `https://api.venice.ai/api/v1` and native `POST /image/generate`.
Refresh `GET /models?type=image` and relevant public styles; retain a dated cache.
Use exact case-sensitive IDs, metadata constraints and capability flags, plus
small documented exceptions where schema presence does not prove effectiveness.
Unknown values remain unknown; no cross-provider contract inference.

Rust validates and assembles the request. Send the prompt exactly as authored,
`safe_mode` explicitly, `enhance_prompt: false`, and only relevant supported
settings. Keep original text separate from any unexpectedly returned revised
text. Default to JSON output (`return_binary: false`) for variants and metadata.
Decode and validate each base64 image; retain original bytes. Do not require an
external image URL or the Venice website's download UI.

The catalog advertises prompt limits from 1,500 to 32,768, while the generic
OpenAPI prompt schema caps at 7,500. Use the model-advertised limit for model
selection and disclose the conflict for longer input. Do not silently truncate
or enforce an invented universal limit. Successful >7,500 acceptance remains an
unverified provider claim until separately tested. Keep a rejected draft intact.

Pixel models use the model's divisor and documented bounds; ratio/tier models
use their own fields. Generic width/height currently max at 1,280, which is not
a global cap on tiered outputs. Do not send width/height to models documented
to reject them. Validate format/transparency combinations. Do not send custom
LoRA uploads or unsupported settings because a generic schema mentions a field.

Generic seed range is -999999999 through 999999999. Documentation describes
omission as random but also gives a schema default of zero. Do not import a
magic random sentinel from another service. The catalog lacks a reliable per-model
seed-support flag: distinguish documented support, verified behavior and unknown.
Requested seed and provider-returned seed are separate; unknown actual seed is
not a reproducibility guarantee. Identical seeds across models are not comparable.

## Response and operation ownership

Create a durable local operation identity before dispatch. The Rust process owns
the request independently of the browser connection. Persist successful outputs
as they become available. A restart cannot recover an undocumented remote job;
an interrupted dispatched operation becomes indeterminate, not automatically retried.

Accept successful, partial, moderated and failed outcomes. A 200 can contain
fewer images than requested; retain `content_policy_violation_count` and the
corresponding headers. All variants refused can return 422. A flagged image may
still be returned. Never assume requested count equals delivered count or that
success means unmoderated output. Preserve available evidence even when some
payload items cannot be decoded, without claiming corrupt bytes are valid images.

Read structured and plain-text error bodies, HTTP status, provider request ID,
documented reason/code, field-validation details, `retryable`, `suggested_prompt`,
`recommended_model` and returned refund information where present. Unknown error
shapes require a generic explanation plus bounded redacted evidence. Header flags
may describe a response collectively; do not invent per-image attribution.

Use one generation attempt per explicit user action. No transparent retries,
redirects carrying credentials, paid fallback model, background regeneration or
automatic repair prompt. A failed body read, timeout or disconnect can have incurred
charges. Retry UI explains that uncertainty. A save retry reuses already-received
bytes. Cancellation must not imply remote cancellation or a refund without evidence.

## Prices, limits and evidence

Choose the applicable quality/resolution matrix before a flat catalog price;
include variants and selected extras. Show estimates as estimates, not actual
charges or guaranteed spending caps. Missing billing/refund data means unknown.
Read rate-limit/retry timing where available; bounded read-only refresh retries
do not authorize generation retries. Size/time bounds must accommodate documented
large outputs and slow generation while limiting resource exhaustion; test those
bounds before claiming supported tiers. Preserve explicit truncation/body-read
failure evidence.

Offline fixtures prove client behavior, not that Venice honors a field. Model
claims, observed responses and app interpretations remain distinguishable in
both UI and saved history. Future provider checks must be bounded and authorized;
do not spend credits merely to refresh documentation.
