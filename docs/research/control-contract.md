# Generation control applicability

Dated implementation evidence, **2026-11-14**, for [issue #3](https://github.com/joeloverbeck/venice-image-studio/issues/3).
Current catalog metadata governs availability, selectable values and prices.
The native schema owns wire types/bounds; explicit control documentation establishes
applicability. Examples establish documented requests, not measured effectiveness.
Unknown or conflicting optional controls remain unavailable. No owner content or
paid model benchmark was used for this mapping.

The [current catalog](https://api.venice.ai/api/v1/models?type=image) matched research
SHA-256 `faaf625846a2639e12ad5a9f1c251ea7993dea942adf3a8c29739c87d01c226e`.
The [native schema](https://raw.githubusercontent.com/veniceai/api-docs/main/swagger.yaml)
matched SHA-256 `c100cced1ce996fe1f7844bb5e27115ff89cc02c74132a045c8e00c0347d4a59`.
The [Venice control reference](https://github.com/veniceai/skills/blob/0f338f021a3ef256bbbd6b7eafb54a5d865021e4/skills/venice-image-generate/SKILL.md)
is pinned at its 2026-10-02 file revision, SHA-256
`fa8db3d8e933f007746ecbb3e5df38d8dbe4a0f6000ed4c64ae0e3d4931461ac`.
It is supplemental provider documentation, not an invoked workflow. Its older
model inventories and refusal examples do not override current catalog/native
response contracts.

| Control | Documented mapping used by the app | Limit or contrary evidence |
| --- | --- | --- |
| Pixel sizing | SD35, three Lustify entries, WAI, Chroma, Z-Image Turbo | Positive width/height ≤1280 and current divisor; divisor presence alone never selects this mode |
| Ratio sizing | Explicit current `aspectRatios`, with optional `resolutions` and `qualities` | Omit pixel fields; a tier is not an exact output dimension |
| Negative prompt | SD35, Lustify SDXL/v7/v8, WAI; Qwen Image 2/2 Pro/3/3 Pro | Chroma/Z ignore it; FLUX.2, Luma and Qwen 2.1 lack support; Wan has conflicting Venice/upstream evidence |
| Steps | SD35, Lustify SDXL/v7/v8, WAI | Integer 1..current maximum; generic metadata on other models is not evidence of effect |
| CFG | SD35 and WAI | 0 < CFG ≤20; dated native examples below, WAI evidence is weaker; Z-Image Turbo does not use CFG |
| Seed | Z-Image Turbo, Seedream V4.5 (`seedream-v4`), Nano Banana 2 | Explicit native examples; −999999999..999999999. Other routes stay unavailable where support is absent/unknown |
| Style preset | SD35, WAI, Qwen Image 2 | Exact native examples; membership in refreshed `/image/styles`; style references are a separate deferred feature |
| Transparency | Current `supportsTransparentBackground: true` | PNG/WebP only; do not transfer support to another family member |
| Web search | Current `supportsWebSearch: true` | Starts false; unknown image surcharge blocks search-enabled dispatch |
| Optimization/thinking | Current `supportsOptimizePromptThinking: true` | Off sends `disable_prompt_optimization_thinking: true`; on sends false. Listed model price remains an estimate, with no separate tariff published |

The pinned Venice reference supplies the explicit negative/steps/seed mappings.
The [native generation guide](https://docs.venice.ai/guides/media/image-generation)
provides SD35 negative-prompt and Qwen Image 2 style examples.
Venice’s [2025-05-29 SD35 example](https://venice.ai/blog/ai-generated-images-meets-privacy-secure-creative-workflows-with-venice-make)
uses CFG 3 and 30 steps. Its [official WAI Postman request](https://www.postman.com/veniceai/venice-ai-workspace/request/qb8geb2/anime-wai?tab=body)
was indexed with CFG 1, Photographic style, a negative prompt and 8 steps; the
client-rendered page itself exposed only its application shell during retrieval.
That weaker WAI provenance is disclosed rather than presented as a live observation.

Contrary sources: [BFL FLUX.2 guide](https://docs.bfl.ai/guides/prompting_guide_flux2),
[Luma FAQ](https://docs.agents.lumalabs.ai/guides/faq/),
[Alibaba text-to-image guide](https://www.alibabacloud.com/help/en/model-studio/text-to-image),
and the [Z-Image Turbo author](https://huggingface.co/Tongyi-MAI/Z-Image-Turbo/discussions/8).
Their exact/version scopes matter; these are not universal family rules.

The current schema warns that web search can cost extra. The older pinned reference
says pricing was unchanged, but the current image catalog supplies no authoritative
surcharge amount. Chat/tool search prices are not image-grounding prices. The app
shows the model base price plus unknown surcharge and prevents that selection from
dispatching. For optimization, the model’s listed base price remains an estimate;
no distinct optimization tariff is published. Enhancement stays false and contributes
no optional enhancement fee.

The app uses quality matrix → resolution matrix → flat generation price precedence.
A missing applicable matrix cell is unknown, never a cheaper fallback. Prompt limits
above 7,500 remain allowed where advertised, with the generic schema conflict
visible and provider acceptance unverified. Requested seed and independently returned
seed remain separate; echoed request fields are not actual-seed evidence.

All 48 practical guidance entries in `src/model-guidance.json` are a literal derived
presentation of [the reviewed model guide](model-guide.md): exact-ID fit/tradeoff
paragraphs, family advice and source links. The source digest and review date travel
with the derived guidance. It is not a runtime availability registry. Live new IDs
remain browseable with explicit guidance gaps. No control is inferred from that prose.
