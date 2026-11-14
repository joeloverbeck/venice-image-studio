# Venice generation model guide

Reference research as of **2026-11-14**, covering all **48** IDs returned by
[`GET /models?type=image`](https://api.venice.ai/api/v1/models?type=image).
Every model's Venice page was reviewed, with primary upstream sources for
family-specific prompting where available. No images were generated or ranked.

Descriptions below are paraphrased vendor positioning. Practical starting points
are editorial suggestions, not verified comparative results. Exact API limits,
prices and feature availability come from Venice, even when an upstream API
offers more. A higher version or price does not establish better results for
the owner's task. Runtime discovery must replace this snapshot when it changes.

## Quality-oriented starting points

| Task | Candidates worth inspecting | Main tradeoff |
| --- | --- | --- |
| Detailed photography and product work | FLUX.2 Max, GPT Image 2.6 Sunburst, Seedream V5.5, Krea v2 Large | Prompt room, price, privacy and unwanted internal expansion differ |
| Posters, typography and graphics | Ideogram V4.5, GPT Image 2.6 Sunburst, Recraft V4/Pro, Qwen Image 3 Pro | Text accuracy still needs visual inspection; premium tiers cost more |
| Very long, structured scene briefs | Nano Banana Pro/2.1; FLUX.3 | Largest advertised capacities, but >7,500 acceptance is not verified here |
| Anime | Anime (WAI); compare general models on the owner's style | WAI's 1,500-character cap remains, despite the native endpoint |
| Painterly work | Venice SD35; general models with a clear medium/style brief | SD35 also retains a 1,500-character cap |
| Models expressly advertised uncensored | Inspect all 14 positive catalog claims below; examples include Chroma, Lustify, WAI, Seedream and Qwen variants | Exact IDs and sources matter; a claim does not guarantee acceptance |
| Fast iteration before a final choice | Z-Image Turbo, Krea 2 Turbo, Qwen Image 2.1 Turbo | Speed/cost positioning is not proof of strongest final quality |

These shortlists are inferences from the linked descriptions. They are optional
starting points, not automatic routing or a ranked leaderboard. Existing catalog
traits (`highest_quality`, `fastest`, `most_uncensored`) are Venice's labels; for
example the `highest_quality` trait on older Qwen Image does not settle a universal
comparison against all newer models.

## Read the entries correctly

- **Private** = Venice claims zero content retention. **Anonymous** = identity
  obscured; upstream content retention is not established. Neither is a censorship
  label or an independently audited guarantee. [Privacy evidence](privacy-and-refusals.md)
- **Uncensored claim not established** means no model-specific assertion found
  in this sweep, not a determination that a model always refuses adult content.
  The saved catalog contains 14 positive `model_spec.uncensored` flags and 34
  omissions, with no false values. Reconciliation on 2026-11-14 corrected the
  earlier prose, which counted only six model-page descriptions; see the
  [claim evidence](privacy-and-refusals.md#model-level-uncensored-evidence).
- Price ranges are USD per generated image across advertised generation settings,
  excluding extras; quality matrices take precedence over resolution-only prices.
  They are neither invoices nor account credit checks.
- Pixel sizing uses the shown divisor and the endpoint's bounds. “Ratio” means
  choose an allowed aspect ratio; “ratio + tiers” also exposes resolution choices.
  Do not translate a tier into a guaranteed exact pixel size.
- Limits over 7,500 conflict with the generic request schema and remain untested.
  No exact tokenizer or per-model seed-support flag was established. Generic
  steps fields on ratio models do not prove adjustable sampling.
- The [factual catalog extract](models-2026-11-14.json) retains exact constraints,
  defaults, prices, flags and source URLs. It excludes copied provider descriptions.

## Prompting sources and applicability

### General brief

Editorial starting point when no exact guide is available: state the subject,
composition, intended medium, lighting, colors and required text clearly. Put
essential requirements first. Longer is useful when it adds constraints; padding
and contradictory adjectives are not a quality setting. Nothing here authorizes
the app to add these details to the owner's prompt. Quoted text and explicit
placement are useful to try for graphics. Exact syntax remains unverified unless
a family section below says otherwise.

### FLUX

Black Forest Labs' [FLUX.2 guide](https://docs.bfl.ai/guides/prompting_guide_flux2)
recommends a natural-language subject/action/style/context structure, with important
elements first. It documents no negative-prompt support: describe wanted content
positively. Its word-count examples are guidance, not Venice limits. Structured
JSON is optional prose, not an app schema. This source applies to FLUX.2; do not
automatically transfer every control or syntax rule to FLUX.3.

### GPT Image

OpenAI's [image prompting guide](https://developers.openai.com/api/docs/guides/image-prompting)
covers the 2.6 and 2.5 Flare/Sunburst distinction: speed-oriented versus more
demanding detail/layout. Use an explicit intended output, subject, composition,
style and constraints; labeled sections can help complex briefs. Quote exact
lettering and state repetitions/placement. The guide also points older models
to shared techniques, with model-specific testing still needed. Venice does not
expose every upstream parameter, so do not add upstream `complexity`, quality
levels or sizing rules absent from its own contract.

### Nano Banana

Google's [image-generation guide](https://ai.google.dev/gemini-api/docs/image-generation)
supports detailed natural-language briefs with context and explicit composition.
Pro is positioned for complex fidelity-sensitive work; 2/2.1 balance capability
and efficiency; Lite favors speed. Use clear text and spatial requirements, not
a claim that more tokens always help. Upstream conversational editing/grounding
features do not imply equivalent Venice support. Only the three non-Lite entries
currently advertise Venice's web-search capability.

### Ideogram

Ideogram's [creation documentation](https://docs.ideogram.ai/create/create)
supports plain-language descriptions with quoted desired lettering. For a poster,
identify hierarchy, text placement, style and colors. These are family techniques;
the Venice V4/V4.5 pages establish version-specific controls but not a mandatory
new prompt language. Do not assume a website's “magic prompt” feature is required.

### Recraft

Recraft's [prompting guide](https://www.recraft.ai/blog/prompt-engineering-guide)
recommends starting from scene and composition, then important details and light.
Shorter briefs support exploration; structured detail gives design control.
Specify the intended design use and visual constraints. Recraft's own vector
products are separate: Venice's generation endpoint documents raster output,
so this app must not advertise SVG generation from the Recraft model name alone.

### Luma

Luma's [Uni-1 prompt guide](https://docs.agents.lumalabs.ai/guides/images/prompt-guide/)
recommends a natural creative brief, spatial specificity and exact quoted text;
negative prompts are not supported upstream. Venice's Uni-1 descriptions mention
web-grounded knowledge, but `supportsWebSearch` is false: that does not create a
user-controlled search toggle. Reference-count differences between upstream and
Venice are another reason to use Venice's contract.

### Qwen and Wan

Alibaba's [text-to-image guide](https://www.alibabacloud.com/help/en/model-studio/text-to-image)
recommends subject, scene, style, lighting and composition. Wan 2.7 and Qwen 2.1
do not support its separate negative-prompt parameter. Newer Qwen routes can
include upstream prompt rewriting; Venice's optional enhancer being off does
not prove every upstream rewrite is disabled. Do not silently import upstream
defaults, 4K modes, thinking or rewritten prompts into Venice requests.

The original [Qwen Image model card](https://huggingface.co/Qwen/Qwen-Image)
emphasizes text rendering, especially Chinese. Exact desired text and layout
are sensible starting points across this family, but original-model behavior
does not prove later version parity.

### Grok

xAI's [image prompting guide](https://docs.x.ai/developers/model-capabilities/images/prompting-guide)
for Imagine 2.0 recommends a clear subject with count/material/framing, then
art direction, lighting and color. Specify a coherent single image and important
crop/focus. Use this as family-level advice for older Grok, not verified parity.
Venice's selected quality settings and privacy claim remain route-specific.

### Seedream

ByteDance's [Seedream 5.5 page](https://seed.bytedance.com/en/seedream5_5)
demonstrates precise natural-language material, lettering and transparency
instructions. Its [release account](https://seed.bytedance.com/en/blog/more-realistic-images-more-stable-editing-across-100-rounds-introducing-seedream-5-5/)
emphasizes realism and editing stability. Those demonstrations do not establish
mandatory syntax or transfer editing guarantees to this generation-only app.
For V4.5 and V5 variants, use the general brief and their own Venice page; no
version-specific required syntax was established in this sweep.

### Krea

The [Krea 2 Turbo model card](https://huggingface.co/krea/Krea-2-Turbo)
uses descriptive scene prose with specific perspective, light and texture.
It supports that style of starting prompt, not mandatory magic words. Exact
prompt guides for Venice's other Krea tiers were not established. Venice
explicitly advertises built-in expansion for Krea 3; warn that short prompts
can acquire extra detail even with the optional Venice enhancer disabled.

### Hunyuan

Tencent's [Instruct model card](https://huggingface.co/tencent/HunyuanImage-3.0-Instruct)
and [prompt notes](https://huggingface.co/tencent/HunyuanImage-3.0-Instruct/blob/main/Hunyuan-Image3.md)
distinguish the Instruct checkpoint's reasoning/self-rewrite from the base
checkpoint. Venice serves Instruct. Write explicit instructions; do not promise
that disabling Venice's enhancer removes internal reasoning or rewriting.
No exposed Venice switch establishing that guarantee was found.

### Z-Image Turbo

The [official author's guidance](https://huggingface.co/Tongyi-MAI/Z-Image-Turbo/discussions/8)
supports detailed prompts and says the distilled Turbo model does not use
negative prompts or CFG. This applies to Turbo, not the non-distilled Z-Image
family. Venice's fixed eight-step metadata should not become a misleading
quality slider. Detailed prose is optional advice, not an automatic rewrite.

### SD35, Chroma, Lustify and WAI

Stability's [SD3.5 introduction](https://stability.ai/news-updates/introducing-stable-diffusion-3-5)
describes diverse style capability; Venice specifically positions its deployment
toward painterly work. Use a clear medium, composition and subject within its
short limit. [Chroma's model card](https://huggingface.co/lodestones/Chroma1-HD)
establishes the architecture/source, but this sweep did not verify a required
Venice-specific syntax or that FLUX.2's negative-prompt rules transfer to Chroma.

The author pages linked by Venice for
[Lustify](https://civitai.com/models/573152/lustify-sdxl-nsfw-checkpoint) and
[WAI](https://civitai.com/models/827184?modelVersionId=1761560) could not be read
through the research browser. Their Venice pages were read. Do not invent
checkpoint trigger words, weighted-tag syntax, samplers or mandatory quality
tags. A concise photographic brief for Lustify and clear anime style/composition
for WAI are editorial starting points, pending exact-checkpoint evidence.

### Muse and MiniMax

Meta's [Muse release](https://ai.meta.com/blog/introducing-muse-image-muse-video-msl)
supports its instruction-following/text-rendering positioning. Use a clear creative
brief; do not assume upstream agentic tools are exposed by Venice. This sweep
did not establish a distinct required prompt format for Muse or MiniMax H3.
MiniMax's Venice page remains the source for the H3 entry and controls; general
MiniMax API documentation is not enough to claim exact H3 prompting guarantees.

## Complete generation inventory

Every following entry links its exact Venice source. Prompting sections above
explicitly distinguish exact-version advice, family advice and research gaps.

### Venice SD35 — `venice-sd35`

Venice’s SD3.5 Large deployment is positioned for painting and artistic interpretation. Its short input limit makes it unsuitable for an unchanged long brief. [Venice model page](https://docs.venice.ai/models/venice-sd35)

- Advertised prompt limit: **1,500 characters**; sizing: **pixels, multiples of 16**; generation estimate: **$0.01**.
- **Private (Venice zero-retention claim)**; uncensored claim not established.
- Prompting: [sd35 chroma lustify and wai](#sd35-chroma-lustify-and-wai). Catalog distinctions: steps default/max: 25/30; provider traits: `eliza-default`.

### Seedream V5.5 — `seedream-v5-5`

The newer Seedream choice emphasizes lifelike materials and lettering. It offers a 1.5K intermediate tier and advertised alpha output; do not conflate generation with the separately named edit model. [Venice model page](https://docs.venice.ai/models/seedream-v5-5)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 1.5K, 2K**; generation estimate: **$0.0255–$0.051**.
- **Anonymous (upstream retention unknown)**; uncensored claim in the saved Venice catalog (`model_spec.uncensored: true`).
- Prompting: [seedream](#seedream). Catalog distinctions: alpha advertised.

### Grok Imagine 2.0 — `grok-imagine-image-2-0`

Second-generation Grok adds selectable low/medium quality and more aspect choices than the older Grok route. It is one of the catalog’s private routes, without an established uncensored guarantee. [Venice model page](https://docs.venice.ai/models/grok-imagine-2-0)

- Advertised prompt limit: **7,500 characters**; sizing: **ratio + 1K, 2K**; generation estimate: **$0.05–$0.1**.
- **Private (Venice zero-retention claim)**; uncensored claim not established.
- Prompting: [grok](#grok). Catalog distinctions: quality: low, medium.

### Krea 3 — `krea-3`

Krea’s newer tier adds 4K and medium/high quality. Built-in prompt expansion is a material tradeoff for an owner who dislikes added content; no disabling guarantee was found. [Venice model page](https://docs.venice.ai/models/krea-3)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.04–$0.15**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [krea](#krea). Catalog distinctions: quality: medium, high.

### Krea 2 Turbo — `krea-2-turbo`

A fast open-weight Krea variant aimed at inexpensive iteration. It has explicit resolution tiers and a private route; the Turbo name does not establish identical output to Krea Large. [Venice model page](https://docs.venice.ai/models/krea-2-turbo)

- Advertised prompt limit: **5,000 characters**; sizing: **ratio + 1K, 2K**; generation estimate: **$0.04–$0.06**.
- **Private (Venice zero-retention claim)**; uncensored claim in the saved Venice catalog (`model_spec.uncensored: true`).
- Prompting: [krea](#krea).

### Flux 2 Pro — `flux-2-pro`

A balance of detailed prompt following and speed in the FLUX.2 family. It costs less than Max but has the same advertised prompt capacity. [Venice model page](https://docs.venice.ai/models/flux-2-pro)

- Advertised prompt limit: **3,000 characters**; sizing: **ratio**; generation estimate: **$0.03**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [flux](#flux).

### Flux 2 Max — `flux-2-max`

FLUX.2’s premium tier targets realism and precision. It costs three times Pro in this snapshot; whether its detail improvement matters needs the owner’s actual prompt. [Venice model page](https://docs.venice.ai/models/flux-2-max)

- Advertised prompt limit: **3,000 characters**; sizing: **ratio**; generation estimate: **$0.09**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [flux](#flux).

### Flux 3 — `flux-3-image`

The newer flagship emphasizes text, layout and high-resolution output with much more prompt room than FLUX.2. Its 4K price is substantial; FLUX.2 prompting/control rules are not automatically exact-version evidence. [Venice model page](https://docs.venice.ai/models/flux-3-image)

- Advertised prompt limit: **32,000 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.12–$0.98**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [flux](#flux).

### GPT Image 2.5 Flare — `gpt-image-2-5-flare`

The speed-oriented 2.5 choice for iterative creation. It shares Venice’s price matrix with 2.5 Sunburst here, so cost alone does not explain their quality/latency distinction. [Venice model page](https://docs.venice.ai/models/gpt-image-2-5-flare)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.03–$0.23**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [gpt image](#gpt-image). Catalog distinctions: quality: low, medium, high; alpha advertised.

### GPT Image 2.5 Sunburst — `gpt-image-2-5-sunburst`

The demanding-detail 2.5 choice, aimed at product and campaign work. Compare against 2.6 on actual useful detail before treating a version increase as a universal improvement. [Venice model page](https://docs.venice.ai/models/gpt-image-2-5-sunburst)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.03–$0.23**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [gpt image](#gpt-image). Catalog distinctions: quality: low, medium, high; alpha advertised.

### GPT Image 2.6 Sunburst — `gpt-image-2-6-sunburst`

The newer fidelity-oriented choice for complex layouts, product images and detailed final work. This is a strong candidate to inspect for the owner’s quality preference, not a measured winner. [Venice model page](https://docs.venice.ai/models/gpt-image-2-6-sunburst)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.05–$0.24**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [gpt image](#gpt-image). Catalog distinctions: quality: low, medium, high; alpha advertised.

### GPT Image 2.6 Flare — `gpt-image-2-6-flare`

The newer faster tier aims to retain useful detail with less latency than Sunburst. Venice currently lists the same prices as 2.6 Sunburst; measure usefulness and waiting time separately. [Venice model page](https://docs.venice.ai/models/gpt-image-2-6-flare)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.05–$0.24**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [gpt image](#gpt-image). Catalog distinctions: quality: low, medium, high; alpha advertised.

### GPT Image 2 — `gpt-image-2`

Earlier GPT generation with typography and fine-detail positioning. Its high-quality tiers are much more expensive than low quality; resolution-only and quality-matrix prices differ slightly. Use the applicable matrix. [Venice model page](https://docs.venice.ai/models/gpt-image-2)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.02–$0.83**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [gpt image](#gpt-image). Catalog distinctions: quality: low, medium, high.

### GPT Image 1.5 — `gpt-image-1-5`

An older GPT choice with only three advertised ratios, less prompt room and a flat price. Alpha output is advertised, but newer tier controls must not be assumed here. [Venice model page](https://docs.venice.ai/models/gpt-image-1-5)

- Advertised prompt limit: **5,000 characters**; sizing: **ratio**; generation estimate: **$0.26**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [gpt image](#gpt-image). Catalog distinctions: alpha advertised.

### Hunyuan Image 3.0 — `hunyuan-image-v3`

Tencent’s reasoning-oriented Instruct deployment is suited to explicit, instruction-heavy composition. Internal self-rewriting is a possible fidelity tradeoff even when Venice enhancement is disabled. [Venice model page](https://docs.venice.ai/models/hunyuan-image-3-0)

- Advertised prompt limit: **3,000 characters**; sizing: **ratio**; generation estimate: **$0.09**.
- **Private (Venice zero-retention claim)**; uncensored claim not established.
- Prompting: [hunyuan](#hunyuan).

### Ideogram V4 — `ideogram-v4`

A typography-oriented option for posters, logos and promotional graphics. It has a simple flat price, without V4.5’s exposed quality/resolution choices. [Venice model page](https://docs.venice.ai/models/ideogram-v4)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio**; generation estimate: **$0.06**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [ideogram](#ideogram).

### Ideogram V4.5 — `ideogram-v4-5`

A newer typography option with selectable quality and 1K/2K tiers. The quality choice changes price strongly, while the captured matrix charges the same at either resolution. [Venice model page](https://docs.venice.ai/models/ideogram-v4-5)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K**; generation estimate: **$0.04–$0.26**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [ideogram](#ideogram). Catalog distinctions: quality: low, medium, high.

### Krea v2 Large — `krea-v2-large`

Krea’s refined-detail/aesthetic tier, positioned above Medium. It advertises three style references with strength control, a capability for later scope rather than a required first-release upload flow. [Venice model page](https://docs.venice.ai/models/krea-v2-large)

- Advertised prompt limit: **5,000 characters**; sizing: **ratio**; generation estimate: **$0.07**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [krea](#krea). Catalog distinctions: style references advertised (later scope).

### Krea v2 Medium — `krea-v2-medium`

The less expensive Krea v2 alternative balancing speed and appearance. It has the same prompt cap and reference capabilities as Large; the catalog does not quantify their quality difference. [Venice model page](https://docs.venice.ai/models/krea-v2-medium)

- Advertised prompt limit: **5,000 characters**; sizing: **ratio**; generation estimate: **$0.04**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [krea](#krea). Catalog distinctions: style references advertised (later scope).

### Luma Uni-1 — `luma-uni-1`

A general high-fidelity option positioned around informed interpretation of the brief. Its web-grounding description does not expose a search switch. Three style references are advertised, without strength control. [Venice model page](https://docs.venice.ai/models/luma-uni-1)

- Advertised prompt limit: **6,000 characters**; sizing: **ratio**; generation estimate: **$0.05**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [luma](#luma). Catalog distinctions: style references advertised (later scope).

### Luma Uni-1 Max — `luma-uni-1-max`

Uni-1’s higher-fidelity tier costs more than twice standard in this snapshot. The prompt and reference limits remain the same; inspect whether the premium changes useful details. [Venice model page](https://docs.venice.ai/models/luma-uni-1-max)

- Advertised prompt limit: **6,000 characters**; sizing: **ratio**; generation estimate: **$0.12**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [luma](#luma). Catalog distinctions: style references advertised (later scope).

### MiniMax H3 Image — `minimax-h3-image`

A general option with broad landscape/portrait ratios and 4K availability. 1K and 2K share the listed price. Optional optimization/thinking is advertised, but no special prompt grammar was established. [Venice model page](https://docs.venice.ai/models/minimax-h3-image)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.04–$0.1**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [muse and minimax](#muse-and-minimax). Catalog distinctions: optimization/thinking advertised.

### Muse Image — `muse-image`

Meta’s model emphasizes instruction fidelity, typography and small details at a low listed flat price. Upstream claims about tool-assisted creation do not mean the Venice route exposes those tools. [Venice model page](https://docs.venice.ai/models/muse-image)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio**; generation estimate: **$0.02**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [muse and minimax](#muse-and-minimax).

### Nano Banana 2 — `nano-banana-2`

A general Gemini-based option with extensive prompt capacity and extreme ratios. It is a plausible long-brief candidate, subject to the unresolved 7,500-character schema conflict. [Venice model page](https://docs.venice.ai/models/nano-banana-2)

- Advertised prompt limit: **32,768 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.1–$0.19**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [nano banana](#nano-banana). Catalog distinctions: web search advertised.

### Nano Banana 2.1 — `nano-banana-2-1`

An updated Nano Banana 2 route with the same captured limit, tier prices and ratios. Newer version alone does not prove better results; use the same original brief for an explicit comparison. [Venice model page](https://docs.venice.ai/models/nano-banana-2-1)

- Advertised prompt limit: **32,768 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.1–$0.19**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [nano banana](#nano-banana). Catalog distinctions: web search advertised.

### Nano Banana Pro — `nano-banana-pro`

The higher-priced Nano Banana tier is positioned for demanding fidelity and complex briefs. It retains the family’s large prompt allowance and extreme ratios; none is a measured success guarantee. [Venice model page](https://docs.venice.ai/models/nano-banana-pro)

- Advertised prompt limit: **32,768 characters**; sizing: **ratio + 1K, 2K, 4K**; generation estimate: **$0.18–$0.35**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [nano banana](#nano-banana). Catalog distinctions: web search advertised.

### Nano Banana 2 Lite — `nano-banana-2-lite`

The efficiency-focused option has a smaller prompt allowance, no exposed resolution tier and no web-search capability flag. Do not copy the Pro/2/2.1 settings wholesale. [Venice model page](https://docs.venice.ai/models/nano-banana-2-lite)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio**; generation estimate: **$0.06**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [nano banana](#nano-banana).

### Recraft V4 — `recraft-v4`

A design-oriented choice for branding, graphics and text. The native Venice response is raster; a vector-like composition request will not make this endpoint return SVG. [Venice model page](https://docs.venice.ai/models/recraft-v4)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio**; generation estimate: **$0.05**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [recraft](#recraft).

### Recraft V4 Pro — `recraft-v4-pro`

The premium Recraft option emphasizes finer product/design detail at a much higher price. The same prompt and ratio options do not establish that the premium is needed for every graphic. [Venice model page](https://docs.venice.ai/models/recraft-v4-pro)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio**; generation estimate: **$0.29**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [recraft](#recraft).

### Seedream V4.5 — `seedream-v4`

The friendly name is Seedream V4.5 although the API ID says v4. Venice specifically calls this deployment uncensored. That claim coexists with anonymous privacy, so upstream retention remains unestablished. [Venice model page](https://docs.venice.ai/models/seedream-v4-5)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio**; generation estimate: **$0.05**.
- **Anonymous (upstream retention unknown)**; uncensored claim in the linked Venice description.
- Prompting: [seedream](#seedream).

### Seedream V5 Lite — `seedream-v5-lite`

A faster Seedream 5.0 variant with a flat price and several ratios. Its own catalog entry carries an uncensored claim; do not inherit V5.5’s transparency capability merely from the family name. [Venice model page](https://docs.venice.ai/models/seedream-v5-lite)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio**; generation estimate: **$0.05**.
- **Anonymous (upstream retention unknown)**; uncensored claim in the saved Venice catalog (`model_spec.uncensored: true`).
- Prompting: [seedream](#seedream).

### Seedream V5 Pro — `seedream-v5-pro`

Seedream 5.0’s higher-end tier exposes 1K/2K and optimization/thinking. It is distinct from the newer V5.5 route; optional optimization stays off unless deliberately selected. [Venice model page](https://docs.venice.ai/models/seedream-v5-pro)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K**; generation estimate: **$0.06–$0.11**.
- **Anonymous (upstream retention unknown)**; uncensored claim in the saved Venice catalog (`model_spec.uncensored: true`).
- Prompting: [seedream](#seedream). Catalog distinctions: optimization/thinking advertised.

### Qwen Image 2 — `qwen-image-2`

The Qwen 2.0 standard tier emphasizes native 2K and lettering. Venice controls it through aspect ratios; its upstream API’s custom-size rules do not authorize width/height here. [Venice model page](https://docs.venice.ai/models/qwen-image-2)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio**; generation estimate: **$0.05**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [qwen and wan](#qwen-and-wan).

### Qwen Image 2 Pro — `qwen-image-2-pro`

A more expensive Qwen 2.0 tier positioned for professional text/detail. Its input limit and ratios match standard, so stronger useful output remains a comparison to make rather than a certainty. [Venice model page](https://docs.venice.ai/models/qwen-image-2-pro)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio**; generation estimate: **$0.1**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [qwen and wan](#qwen-and-wan).

### Wan 2.7 — `wan-2-7-text-to-image`

The standard Wan 2.7 text-to-image route emphasizes instruction understanding. It has a shorter prompt allowance than many text-oriented alternatives; do not import Alibaba’s larger direct-API allowance. [Venice model page](https://docs.venice.ai/models/wan-2-7-image)

- Advertised prompt limit: **3,000 characters**; sizing: **ratio**; generation estimate: **$0.0375**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [qwen and wan](#qwen-and-wan).

### Wan 2.7 Pro — `wan-2-7-pro-text-to-image`

The more expensive Wan 2.7 generation tier emphasizes detail. Alibaba’s direct API supports additional modes, but this Venice snapshot does not expose a 4K tier or those extra controls. [Venice model page](https://docs.venice.ai/models/wan-2-7-pro)

- Advertised prompt limit: **3,000 characters**; sizing: **ratio**; generation estimate: **$0.09375**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [qwen and wan](#qwen-and-wan).

### Grok Imagine — `grok-imagine-image`

The older Grok route emphasizes photographic output and short waits at lower prices than 2.0. Its catalog is private, but this sweep did not establish an uncensored claim. [Venice model page](https://docs.venice.ai/models/grok-imagine-image)

- Advertised prompt limit: **7,500 characters**; sizing: **ratio + 1K, 2K**; generation estimate: **$0.03–$0.04**.
- **Private (Venice zero-retention claim)**; uncensored claim not established.
- Prompting: [grok](#grok).

### Lustify SDXL — `lustify-sdxl`

A community SDXL photographic checkpoint explicitly advertised for uncensored adult imagery. Its 1,500-character cap remains a significant limitation for this owner’s long-prompt workflow. [Venice model page](https://docs.venice.ai/models/lustify-sdxl)

- Advertised prompt limit: **1,500 characters**; sizing: **pixels, multiples of 8**; generation estimate: **$0.01**.
- **Private (Venice zero-retention claim)**; uncensored claim in the linked Venice description.
- Prompting: [sd35 chroma lustify and wai](#sd35-chroma-lustify-and-wai). Catalog distinctions: steps default/max: 20/50.

### Lustify v7 — `lustify-v7`

Version 7 of the adult-oriented photographic checkpoint has Venice’s most_uncensored trait. The trait is promotional metadata, not evidence that every prompt will succeed. [Venice model page](https://docs.venice.ai/models/lustify-v7)

- Advertised prompt limit: **1,500 characters**; sizing: **pixels, multiples of 8**; generation estimate: **$0.01**.
- **Private (Venice zero-retention claim)**; uncensored claim in the linked Venice description.
- Prompting: [sd35 chroma lustify and wai](#sd35-chroma-lustify-and-wai). Catalog distinctions: steps default/max: 20/50; provider traits: `most_uncensored`.

### Lustify v8 — `lustify-v8`

Version 8 retains the same price and short prompt cap as v7; its listed default steps rise from 20 to 30. No comparative quality or refusal-rate benchmark was performed. [Venice model page](https://docs.venice.ai/models/lustify-v8)

- Advertised prompt limit: **1,500 characters**; sizing: **pixels, multiples of 8**; generation estimate: **$0.01**.
- **Private (Venice zero-retention claim)**; uncensored claim in the linked Venice description.
- Prompting: [sd35 chroma lustify and wai](#sd35-chroma-lustify-and-wai). Catalog distinctions: steps default/max: 30/50; provider traits: `most_uncensored`.

### Qwen Image — `qwen-image`

The original Qwen route is positioned around faithful prompts and Chinese/other text. Venice marks it highest_quality, despite newer family options; treat the trait as attributed metadata, not a settled ranking. [Venice model page](https://docs.venice.ai/models/qwen-image)

- Advertised prompt limit: **2,048 characters**; sizing: **ratio**; generation estimate: **$0.03**.
- **Anonymous (upstream retention unknown)**; uncensored claim not established.
- Prompting: [qwen and wan](#qwen-and-wan). Catalog distinctions: provider traits: `highest_quality`.

### Anime (WAI) — `wai-Illustrious`

A dedicated anime checkpoint with an explicit uncensored adult-content claim. Its source is Illustrious XL; exact checkpoint prompt tags were not verified. It is a short-prompt specialist. [Venice model page](https://docs.venice.ai/models/anime)

- Advertised prompt limit: **1,500 characters**; sizing: **pixels, multiples of 16**; generation estimate: **$0.01**.
- **Private (Venice zero-retention claim)**; uncensored claim in the linked Venice description.
- Prompting: [sd35 chroma lustify and wai](#sd35-chroma-lustify-and-wai). Catalog distinctions: steps default/max: 25/30.

### Z-Image Turbo — `z-image-turbo`

A distilled fast model with Venice’s default/fastest traits and a low price. It offers more prompt room than the 1,500-character specialists; negative prompt and CFG controls would be misleading. [Venice model page](https://docs.venice.ai/models/z-image-turbo)

- Advertised prompt limit: **7,500 characters**; sizing: **pixels, multiples of 8**; generation estimate: **$0.01**.
- **Private (Venice zero-retention claim)**; uncensored claim not established.
- Prompting: [z image turbo](#z-image-turbo). Catalog distinctions: steps default/max: 8/8; provider traits: `default`, `fastest`.

### Chroma — `chroma`

A community FLUX.1-derived photographic model that Venice explicitly calls uncensored. Its low price, private route and 7,500-character allowance are a distinctive combination, without quality or acceptance guarantees. [Venice model page](https://docs.venice.ai/models/chroma)

- Advertised prompt limit: **7,500 characters**; sizing: **pixels, multiples of 8**; generation estimate: **$0.01**.
- **Private (Venice zero-retention claim)**; uncensored claim in the linked Venice description.
- Prompting: [sd35 chroma lustify and wai](#sd35-chroma-lustify-and-wai). Catalog distinctions: steps default/max: 10/10.

### Qwen Image 3 — `qwen-image-3`

The newer standard Qwen generation tier emphasizes text at low cost, with 1K and 2K equally priced here. Optimization/thinking is supported but is a deliberate option, not an invisible default. [Venice model page](https://docs.venice.ai/models/qwen-image-3)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K**; generation estimate: **$0.04**.
- **Anonymous (upstream retention unknown)**; uncensored claim in the saved Venice catalog (`model_spec.uncensored: true`).
- Prompting: [qwen and wan](#qwen-and-wan). Catalog distinctions: optimization/thinking advertised.

### Qwen Image 3 Pro — `qwen-image-3-pro`

The newer Pro tier emphasizes detailed prompt adherence and text. Its 2K tier costs more than 1K, unlike standard Qwen Image 3; set expectations from the selected matrix. [Venice model page](https://docs.venice.ai/models/qwen-image-3-pro)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K**; generation estimate: **$0.05–$0.09**.
- **Anonymous (upstream retention unknown)**; uncensored claim in the saved Venice catalog (`model_spec.uncensored: true`).
- Prompting: [qwen and wan](#qwen-and-wan). Catalog distinctions: optimization/thinking advertised.

### Qwen Image 2.1 Turbo — `qwen-image-2-1-turbo`

The fast Qwen 2.1 tier emphasizes efficient iteration at a low equal price for 1K/2K. Its upstream family does not support a separate negative prompt, despite generic Venice schema fields. [Venice model page](https://docs.venice.ai/models/qwen-image-2-1-turbo)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K**; generation estimate: **$0.02**.
- **Anonymous (upstream retention unknown)**; uncensored claim in the saved Venice catalog (`model_spec.uncensored: true`).
- Prompting: [qwen and wan](#qwen-and-wan). Catalog distinctions: optimization/thinking advertised.

### Qwen Image 2.1 Pro — `qwen-image-2-1-pro`

The fuller-capacity Qwen 2.1 route trades price for capability relative to Turbo. Its generation controls are separate from the reference limits advertised for the edit sibling. [Venice model page](https://docs.venice.ai/models/qwen-image-2-1-pro)

- Advertised prompt limit: **10,000 characters**; sizing: **ratio + 1K, 2K**; generation estimate: **$0.05**.
- **Anonymous (upstream retention unknown)**; uncensored claim in the saved Venice catalog (`model_spec.uncensored: true`).
- Prompting: [qwen and wan](#qwen-and-wan). Catalog distinctions: optimization/thinking advertised.

## Snapshot provenance and limits

The catalog was captured at `2026-11-14T09:55:49.170302+00:00` with no
credential. Original response SHA-256: `faaf625846a2639e12ad5a9f1c251ea7993dea942adf3a8c29739c87d01c226e`.
The linked JSON is an extract of factual metadata, not a runtime model registry
or an application schema. Model pages and upstream sources were read on the same
research date. Author pages that could not be read and version-specific gaps are
identified above; generic advice must not be relabeled as tested model expertise.
