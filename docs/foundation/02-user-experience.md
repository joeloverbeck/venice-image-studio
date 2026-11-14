# User experience

## Compose and choose

Keep a spacious multiline prompt editor beside the selected model, essential
controls and result. Preserve drafts through ordinary navigation and failures;
never clear text on model switches. Show a live character count and the selected
model's advertised limit. There is no global 1,500-character cap. Count Unicode
scalar values consistently in Rust and TypeScript, label this convention in help,
and retain the uncertainty about Venice's boundary counting. Do not claim an
exact token count or derive billing from a characters-to-tokens heuristic.

Offer searchable models with a compact comparison of intended strengths, prompt
capacity, sizing, price estimate, availability, privacy and content-policy claims.
Separate vendor descriptions, cited prompting guidance and untested application
recommendations. The [model guide](../research/model-guide.md) is the starting
research, not a permanent availability list. Quality-oriented task choices should
help the owner select without pretending a universal strongest model exists.
Remember an explicit selection; never silently substitute another model.

Every model has an accessible detail view: exact ID, friendly name, dated sources,
practical fit, tradeoffs, prompting advice and whether advice applies to the exact
version or only its family. New or unknown models remain browseable without
invented advice. Show refresh failures and stale catalog age. Removed models stay
identifiable in history but cannot be silently replaced when reusing a recipe.

Display separate, filterable claims for uncensored/adult-oriented positioning,
Venice's privacy mode, upstream content retention, and any specifically supported
additional protection. Attribute each claim and show unknowns. `private` means
Venice claims zero content retention; `anonymized` does not establish upstream
non-retention. “Uncensored” never means guaranteed acceptance. Do not infer
censorship from privacy, open weights, vendor nationality or safe-mode settings.

## Controls and generation

Use width/height inputs and aspect presets for pixel-sized models; use supported
aspect ratios and resolution tiers for other models. Explain the difference and
show actual returned pixel dimensions. On a model switch, identify incompatible
settings and require a visible valid choice instead of silently changing intent.

Safe mode is visibly off on first use; later explicit changes persist and remain
visible before dispatch. Prompt enhancement is explicitly disabled. Disclose
known model-internal expansion separately; don't promise the app can disable it.
Unsupported or uncertain controls must not look effective. Provide random versus
explicit seed and reuse where supported, without promising identical regeneration.

Show supported format, variants, quality and useful advanced controls progressively;
keep the basic path simple. Negative prompts, steps, CFG, style presets and alpha
output need model-specific evidence. Styles start unselected. Web search and
optional optimization/thinking start off, require explicit supported selection,
and must not be presented as guaranteed prompt fidelity. Reference-image editing
and its controls are deferred.

Before Generate, show requested count, model, size, safe-mode state and estimated
price with its age/uncertainty. Generate is an explicit spending action. Start
with one image and one in-flight generation, rather than a hidden batch. Prevent
double submission; a browser reload reconnects to the same local operation.
Use honest progress such as “Waiting for Venice,” not invented percentages.

## Inspect, keep and reuse

Show returned images in the app immediately with saving status. Provide fit-to-view,
100% inspection, pan/zoom, fullscreen, image dimensions and an original-byte
download. Keep the prompt/settings reachable without obscuring the image. Basic
keyboard operation, visible focus, readable text and large-image responsiveness
are part of usability. A thumbnail is not the original download.

The library shows saved results and failed/partial attempts, with basic search
by prompt/model and date ordering. Reopen a result, inspect its recipe, and reuse
it into an editable draft. Reuse is not a paid call. Preserve the old recipe when
the draft changes. Do not delete images automatically or tie library survival to
the source checkout. Destructive library actions, if added, must be explicit.

## Explain refusals and failures

Preserve full available provider explanations, codes, field errors, moderation
flags, omitted-variant counts, retry advice and suggestions in an expandable
details view, alongside a concise plain-language summary. Render provider text
as untrusted text. Keep unknown structured details inspectable with secret
redaction and explicit capture limits, rather than discarding unfamiliar shapes.

Distinguish explicit refusal, partial success, blurred/flagged output, invalid
settings, access/balance problems, throttling, provider failure, ambiguous
interruption and local saving failure. Do not label every 422 a policy refusal
or every dark image censorship. If the provider supplies no reason, say so;
never invent hidden policy categories or infer that erotic content caused a
particular rejection. Preserve returned images even when other variants fail.

Provider-suggested prompts and models are advice to inspect, never automatic
changes. `retryable` is provider advice, not permission for an automatic retry.
There is no refusal-bypassing rewrite or automatic fallback. Show any documented
refund result, otherwise label billing/refund status unknown. Report details
without extra moralizing or falsely assuring that safe mode off prevents refusal.
