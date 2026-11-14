//! Venice generation contracts and preflight. Catalog claims are provider evidence,
//! not observations that a model honors a control. Sources: docs/research/model-guide.md
//! and the native image contract, checked 2026-11-14.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const REQUEST_SOURCE: &str = "https://docs.venice.ai/api-reference/endpoint/image/generate";
const PIXEL_MODELS: &[&str] = &[
    "venice-sd35",
    "lustify-sdxl",
    "lustify-v7",
    "lustify-v8",
    "wai-Illustrious",
    "z-image-turbo",
    "chroma",
];

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub sizing: String,
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: Option<String>,
    pub resolution: Option<String>,
    pub quality: Option<String>,
    pub format: String,
    pub variants: u32,
    pub seed: Option<i64>,
    pub negative_prompt: Option<String>,
    pub steps: Option<u32>,
    pub cfg_scale: Option<f64>,
    pub style_preset: Option<String>,
    pub transparent_background: bool,
    pub web_search: bool,
    pub optimize_prompt: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            sizing: "pixels".into(),
            width: 1024,
            height: 1024,
            aspect_ratio: None,
            resolution: None,
            quality: None,
            format: "png".into(),
            variants: 1,
            seed: None,
            negative_prompt: None,
            steps: None,
            cfg_scale: None,
            style_preset: None,
            transparent_background: false,
            web_search: false,
            optimize_prompt: false,
        }
    }
}
impl Settings {
    pub fn from_request(request: &Value) -> Result<Self, String> {
        let mut value = request.clone();
        let object = value.as_object_mut().ok_or("This saved request is not an object. Inspect the original recipe; no settings were substituted.")?;
        if !request["variants"]
            .as_u64()
            .is_some_and(|n| (1..=4).contains(&n))
            || request["enhance_prompt"] != false
        {
            return Err("This recipe requests unsupported variants or enhancement. Inspect its original request; no settings were substituted.".into());
        }
        if request.get("aspect_ratio").is_none()
            && (request.get("width").is_none() || request.get("height").is_none())
        {
            return Err(
                "This recipe has no complete explicit sizing choice; no settings were substituted."
                    .into(),
            );
        }
        for key in [
            "model",
            "prompt",
            "safe_mode",
            "enhance_prompt",
            "return_binary",
            "embed_exif_metadata",
        ] {
            object.remove(key);
        }
        object.insert(
            "sizing".into(),
            json!(if request.get("aspect_ratio").is_some() {
                "ratio"
            } else {
                "pixels"
            }),
        );
        let optional_bool = |value: Option<Value>, default| -> Result<bool, String> {
            match value {
                None => Ok(default),
                Some(value) => value.as_bool().ok_or(
                    "This recipe has a malformed optional setting; no settings were substituted."
                        .into(),
                ),
            }
        };
        let web = optional_bool(object.remove("enable_web_search"), false)?;
        let optimize = !optional_bool(object.remove("disable_prompt_optimization_thinking"), true)?;
        object.insert("web_search".into(), json!(web));
        object.insert("optimize_prompt".into(), json!(optimize));
        serde_json::from_value(value).map_err(|_| "This recipe contains settings the current app cannot represent. Inspect its original request; no settings were substituted.".into())
    }
}
fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}
pub fn contract(model: &Value) -> Value {
    let spec = &model["model_spec"];
    let constraints = &spec["constraints"];
    let id = model["id"].as_str().unwrap_or("");
    let ratios = strings(&constraints["aspectRatios"]);
    let mode = if PIXEL_MODELS.contains(&id) && ratios.is_empty() {
        "pixels"
    } else if !ratios.is_empty() {
        "ratio"
    } else {
        "unknown"
    };
    let resolutions = strings(&constraints["resolutions"]);
    let qualities = strings(&constraints["qualities"]);
    let adjustable = [
        "venice-sd35",
        "lustify-sdxl",
        "lustify-v7",
        "lustify-v8",
        "wai-Illustrious",
    ]
    .contains(&id);
    let negative = if adjustable
        || [
            "qwen-image-2",
            "qwen-image-2-pro",
            "qwen-image-3",
            "qwen-image-3-pro",
        ]
        .contains(&id)
    {
        "documented"
    } else if ["wan-2-7-text-to-image", "wan-2-7-pro-text-to-image"].contains(&id) {
        "conflicting"
    } else if [
        "chroma",
        "z-image-turbo",
        "flux-2-pro",
        "flux-2-max",
        "luma-uni-1",
        "luma-uni-1-max",
        "qwen-image-2-1-turbo",
        "qwen-image-2-1-pro",
    ]
    .contains(&id)
    {
        "unsupported"
    } else {
        "unknown"
    };
    let seed = if ["z-image-turbo", "seedream-v4", "nano-banana-2"].contains(&id) {
        "documented"
    } else if [
        "luma-uni-1",
        "luma-uni-1-max",
        "grok-imagine-image",
        "seedream-v5-pro",
        "nano-banana-pro",
        "muse-image",
        "recraft-v4",
        "recraft-v4-pro",
    ]
    .contains(&id)
    {
        "unsupported"
    } else {
        "unknown"
    };
    let steps = if adjustable && constraints["steps"]["max"].as_u64().is_some_and(|n| n > 0) {
        "documented"
    } else if ["chroma", "z-image-turbo"].contains(&id) {
        "unsupported"
    } else {
        "unknown"
    };
    let cfg = if ["venice-sd35", "wai-Illustrious"].contains(&id) {
        "documented"
    } else if id == "z-image-turbo" {
        "unsupported"
    } else {
        "unknown"
    };
    let style = if ["venice-sd35", "wai-Illustrious", "qwen-image-2"].contains(&id) {
        "documented"
    } else {
        "unknown"
    };
    let divisor = constraints["widthHeightDivisor"]
        .as_u64()
        .filter(|n| *n > 0);
    let presets: Vec<Value> = [
        ("Square 1:1", 1024u64, 1024u64),
        ("Portrait 3:4", 768, 1024),
        ("Landscape 4:3", 1024, 768),
        ("Wide 16:9", 1280, 720),
    ]
    .into_iter()
    .filter(|(_, w, h)| divisor.is_some_and(|n| w % n == 0 && h % n == 0))
    .map(|(label, width, height)| json!({"label":label,"width":width,"height":height}))
    .collect();
    let defaults = Settings {
        sizing: mode.into(),
        aspect_ratio: constraints["defaultAspectRatio"]
            .as_str()
            .filter(|s| ratios.iter().any(|r| r == s))
            .map(String::from),
        resolution: constraints["defaultResolution"]
            .as_str()
            .filter(|s| resolutions.iter().any(|r| r == s))
            .map(String::from),
        quality: constraints["defaultQuality"]
            .as_str()
            .filter(|s| qualities.iter().any(|r| r == s))
            .map(String::from),
        ..Settings::default()
    };
    json!({"sizing":mode,"pixel_max":1280,"variants_min":1,"variants_max":4,"divisor":constraints["widthHeightDivisor"],
        "aspect_ratios":ratios,"resolutions":resolutions,"qualities":qualities,"formats":["png","jpeg","webp"],
        "seed":seed,"negative_prompt":negative,"steps":steps,"cfg_scale":cfg,"style_preset":style,
        "steps_max":constraints["steps"]["max"],"steps_default":constraints["steps"]["default"],"pixel_presets":presets,
        "control_sources":["https://github.com/veniceai/skills/blob/0f338f021a3ef256bbbd6b7eafb54a5d865021e4/skills/venice-image-generate/SKILL.md","https://docs.venice.ai/guides/media/image-generation","https://venice.ai/blog/ai-generated-images-meets-privacy-secure-creative-workflows-with-venice-make","https://www.postman.com/veniceai/venice-ai-workspace/request/qb8geb2/anime-wai?tab=body"],
        "control_notes": {"negative_prompt":"Wan has conflicting Venice/upstream guidance. Chroma and Z ignore this field; other unsupported/unknown mappings remain unavailable.","steps":"Explicit current native support exists for SD35, Lustify and WAI only; generic steps metadata elsewhere is not evidence of effect.","cfg_scale":"SD35 is supported by a dated native example; WAI by weaker indexed official Postman evidence. Neither is measured effectiveness.","style_preset":"Exact native examples establish SD35, WAI and Qwen Image 2. Style references are a separate deferred capability.","seed":"Explicit positive native examples establish Z-Image Turbo, Seedream V4.5 and Nano Banana 2. Unknown routes remain unavailable."},
        "transparent_background":constraints["supportsTransparentBackground"]==true,
        "web_search":spec["supportsWebSearch"]==true,"optimize_prompt":spec["supportsOptimizePromptThinking"]==true,
        "defaults":defaults,"source":REQUEST_SOURCE,"reviewed_at":"2026-11-14",
        "limitations":["Controls describe documented request support, not verified effectiveness or identical regeneration.",
            "Seed omitted: documentation says random, while the generic schema defaults to zero. Actual seed is unknown unless separately returned.",
            "Enhancement is disabled. Model-internal rewriting may still occur."]})
}
fn money(value: &Value) -> Option<f64> {
    value.as_f64().filter(|p| p.is_finite() && *p >= 0.0)
}
pub fn estimate(catalog: &Value, model: &Value, settings: &Settings) -> Result<Value, String> {
    let pricing = &model["model_spec"]["pricing"];
    let (value, basis) = if !pricing["quality"].is_null() {
        let r = settings
            .resolution
            .as_deref()
            .ok_or("Pricing contract changed or needs a resolution choice.")?;
        let q = settings
            .quality
            .as_deref()
            .ok_or("Choose quality for the applicable price matrix.")?;
        (&pricing["quality"][r][q]["usd"], format!("quality.{r}.{q}"))
    } else if !pricing["resolutions"].is_null() {
        let r = settings
            .resolution
            .as_deref()
            .ok_or("The pricing contract changed or needs a resolution choice.")?;
        (
            &pricing["resolutions"][r]["usd"],
            format!("resolutions.{r}"),
        )
    } else {
        (&pricing["generation"]["usd"], "generation".into())
    };
    let base = money(value).ok_or(
        "No usable estimate is available for these choices; refresh the catalog before spending.",
    )?;
    let mut extras = Vec::new();
    if settings.optimize_prompt {
        extras.push(json!({"name":"Optional optimization/thinking","pricing":"Model base-price estimate; no separate optimization tariff is published.","separate_usd":Value::Null}));
    }
    let total = base * f64::from(settings.variants);
    let mut estimate = json!({"usd":total,"base_usd":total,"per_image_usd":base,"requested_count":settings.variants,"extras":extras,"basis":basis,"source":catalog["source"],"fetched_at":catalog["fetched_at"],"kind":"estimate, not charge or cap","warning":"Provider metadata may be stale; actual charge and refund are unknown."});
    if settings.web_search {
        estimate.as_object_mut().unwrap().remove("usd");
        estimate["extras"].as_array_mut().unwrap().push(json!({"name":"Web search","usd":Value::Null,"pricing":"Unknown image-search surcharge: current schema warns of extra cost; an older provider reference says unchanged pricing."}));
        estimate["unavailable_reason"] = json!(
            "Web search has no current authoritative surcharge amount. Base price is shown; generation with this extra is unavailable until a complete estimate is established."
        );
    }
    Ok(estimate)
}
pub fn prepare(
    catalog: &Value,
    model: &Value,
    styles: &Value,
    prompt: &str,
    safe: bool,
    s: &Settings,
) -> (Vec<String>, Value, Value) {
    let mut errors = Vec::new();
    let c = contract(model);
    if model["model_spec"]["offline"] != false {
        errors.push("Selected model availability is unknown or offline.".into())
    }
    let limit = model["model_spec"]["constraints"]["promptCharacterLimit"].as_u64();
    match limit {
        Some(limit) if !prompt.is_empty() && prompt.chars().count() as u64 <= limit => {}
        Some(limit) => errors.push(format!(
            "Prompt must contain 1–{limit} Unicode scalar characters."
        )),
        None => errors.push("Selected model's prompt limit is unknown.".into()),
    }
    if !(1..=4).contains(&s.variants) {
        errors.push("Choose 1–4 images per attempt, as documented for JSON output.".into());
    }
    let mut request = json!({"model":model["id"],"prompt":prompt,"safe_mode":safe,"enhance_prompt":false,"return_binary":false,"embed_exif_metadata":false,"variants":s.variants,"format":s.format});
    if c["sizing"] == "unknown" {
        errors.push("Required sizing contract is unknown. This model remains browsable; generation is unavailable.".into())
    } else if c["sizing"] != s.sizing {
        errors.push(format!(
            "Sizing choice {} is incompatible with this model. Explicitly choose {} settings.",
            s.sizing,
            c["sizing"].as_str().unwrap()
        ))
    } else if s.sizing == "pixels" {
        let divisor = c["divisor"].as_u64().filter(|n| *n > 0);
        if divisor.is_none_or(|n| {
            s.width == 0
                || s.height == 0
                || s.width > 1280
                || s.height > 1280
                || !(s.width as u64).is_multiple_of(n)
                || !(s.height as u64).is_multiple_of(n)
        }) {
            errors.push("Pixel dimensions must be positive, at most 1280, and multiples of the model divisor.".into())
        }
        request["width"] = json!(s.width);
        request["height"] = json!(s.height);
        if s.aspect_ratio.is_some() {
            errors.push(
                "Aspect-ratio field is incompatible with pixel sizing; clear it explicitly.".into(),
            )
        }
    } else {
        if let Some(ratio) = &s.aspect_ratio {
            if strings(&c["aspect_ratios"]).contains(ratio) {
                request["aspect_ratio"] = json!(ratio)
            } else {
                errors.push("Choose an advertised aspect ratio.".into())
            }
        } else {
            errors.push("Choose an advertised aspect ratio.".into())
        }
    }
    for (name, selected, options) in [
        ("resolution", &s.resolution, "resolutions"),
        ("quality", &s.quality, "qualities"),
    ] {
        let choices = strings(&c[options]);
        match selected {
            Some(value) if choices.contains(value) => request[name] = json!(value),
            None if choices.is_empty() => {}
            _ => errors.push(format!(
                "Choose a supported {name}, or clear the incompatible previous choice."
            )),
        }
    }
    if !strings(&c["formats"]).contains(&s.format) {
        errors.push("Choose PNG, JPEG or WebP format.".into())
    }
    let values = [
        ("seed", s.seed.map(Value::from)),
        (
            "negative_prompt",
            s.negative_prompt.clone().map(Value::from),
        ),
        ("steps", s.steps.map(Value::from)),
        ("cfg_scale", s.cfg_scale.map(Value::from)),
        ("style_preset", s.style_preset.clone().map(Value::from)),
    ];
    for (name, value) in values {
        if let Some(value) = value {
            if c[name] != "documented" {
                errors.push(format!(
                    "{name} is {} for this model; clear that choice. {}",
                    c[name].as_str().unwrap_or("unknown"),
                    c["control_notes"][name].as_str().unwrap_or("")
                ));
                continue;
            }
            let valid = match name {
                "seed" => s
                    .seed
                    .is_some_and(|n| (-999999999..=999999999).contains(&n)),
                "negative_prompt" => s
                    .negative_prompt
                    .as_ref()
                    .is_some_and(|s| s.chars().count() <= 7500),
                "steps" => s
                    .steps
                    .is_some_and(|n| n > 0 && u64::from(n) <= c["steps_max"].as_u64().unwrap_or(0)),
                "cfg_scale" => s
                    .cfg_scale
                    .is_some_and(|n| n.is_finite() && n > 0.0 && n <= 20.0),
                "style_preset" => value
                    .as_str()
                    .is_some_and(|v| strings(&styles["data"]).iter().any(|s| s == v)),
                _ => false,
            };
            if valid {
                request[name] = value
            } else {
                errors.push(format!(
                    "{name} is outside its documented range or current advertised choices."
                ))
            }
        }
    }
    if s.transparent_background {
        if c["transparent_background"] == true && ["png", "webp"].contains(&s.format.as_str()) {
            request["transparent_background"] = json!(true)
        } else {
            errors.push("Transparency needs advertised support and PNG or WebP.".into())
        }
    }
    for (control, enabled, field, negative) in [
        ("web_search", s.web_search, "enable_web_search", false),
        (
            "optimize_prompt",
            s.optimize_prompt,
            "disable_prompt_optimization_thinking",
            true,
        ),
    ] {
        if c[control] == true {
            request[field] = json!(enabled ^ negative)
        } else if enabled {
            errors.push(format!(
                "{control} is not advertised for this model; switch it off explicitly."
            ))
        }
    }
    let estimate =
        estimate(catalog, model, s).unwrap_or_else(|reason| json!({"unavailable_reason":reason}));
    if let Some(reason) = estimate["unavailable_reason"].as_str() {
        errors.push(reason.into())
    }
    (errors, request, estimate)
}

pub fn guidance(id: &str) -> Value {
    static GUIDE: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    let guide = GUIDE.get_or_init(|| {
        serde_json::from_str(include_str!("model-guidance.json")).expect("bundled guidance")
    });
    let mut g = guide["models"][id].clone();
    if g.is_null() {
        return json!({"status":"unknown","fit":"No reviewed model-specific guidance is available. Inspect current metadata; do not infer quality or supported controls from the name.","advice":"Editorial starting point: state subject, composition, medium, lighting and required text. This is not tested advice for this model.","sources":[],"reviewed_at":Value::Null});
    }
    g["status"] = json!("reviewed");
    g["reviewed_at"] = guide["reviewed_at"].clone();
    g["derived_from"] = guide["derived_from"].clone();
    g["source_sha256"] = guide["source_sha256"].clone();
    g
}
fn price_rows(pricing: &Value) -> Vec<Value> {
    let mut rows = Vec::new();
    if let Some(resolutions) = pricing["quality"].as_object() {
        for (resolution, qualities) in resolutions {
            if let Some(qualities) = qualities.as_object() {
                for (quality, price) in qualities {
                    rows.push(json!({"choice":format!("{resolution} · {quality}"),"usd":money(&price["usd"])}));
                }
            }
        }
    } else if let Some(resolutions) = pricing["resolutions"].as_object() {
        for (resolution, price) in resolutions {
            rows.push(json!({"choice":resolution,"usd":money(&price["usd"])}));
        }
    } else {
        rows.push(json!({"choice":"One image","usd":money(&pricing["generation"]["usd"])}));
    }
    rows
}
pub fn describe(model: &Value, availability: &str, fetched: &Value) -> Value {
    let id = model["id"].as_str().unwrap_or("");
    let spec = &model["model_spec"];
    let g = guidance(id);
    let prices = price_rows(&spec["pricing"]);
    let amounts: Vec<f64> = prices
        .iter()
        .filter_map(|row| row["usd"].as_f64())
        .collect();
    let price_summary = if amounts.is_empty() {
        "price unknown".into()
    } else {
        let min = amounts.iter().copied().fold(f64::INFINITY, f64::min);
        let max = amounts.iter().copied().fold(0.0, f64::max);
        if min == max {
            format!("${min} USD")
        } else {
            format!("${min}–${max} USD")
        }
    };
    let privacy = match spec["privacy"].as_str() {
        Some("private") => "private",
        Some("anonymized") => "anonymized",
        _ => "unknown",
    };
    let uncensored = match spec["uncensored"].as_bool() {
        Some(true) => "asserted",
        Some(false) if g["dated_uncensored"] == true => "conflicting",
        Some(false) => "not_asserted",
        None => "unknown",
    };
    let claim = |state: &str, raw: Value, meaning: &str| json!({"state":state,"raw":raw,"meaning":meaning,"source":crate::provider::CATALOG_SOURCE,"policy_source":"https://docs.venice.ai/overview/privacy","policy_reviewed_at":"2026-11-14","fetched_at":fetched});
    json!({"id":id,"name":spec["name"].as_str().unwrap_or(id),"availability":availability,
        "prompt_limit":spec["constraints"]["promptCharacterLimit"],"guidance":g,"contract":contract(model),
        "price_rows":prices,"price_summary":price_summary,"price_fetched_at":fetched,"vendor_description":spec["description"],"vendor_traits":spec["traits"],"pricing":spec["pricing"],
        "claims":{
            "uncensored":claim(uncensored,spec["uncensored"].clone(),"Venice positioning, not guaranteed acceptance. Missing flags remain unknown; dated guide assertions may differ."),
            "privacy":claim(privacy,spec["privacy"].clone(),"Venice privacy assertion, not independently audited. Anonymized hides identity, not content from the upstream provider."),
            "upstream_retention":claim(if privacy=="private"{"zero_retention_claim"}else{"unknown"},spec["privacy"].clone(),"Venice says private routes have contract-enforced zero content retention at the processing layer. Anonymized does not establish upstream retention or training terms."),
            "additional_protection":{"state":"unknown","meaning":"No image-specific E2EE, TEE, residency or additional no-training guarantee established. Text-model protection claims do not transfer to images.","source":"https://docs.venice.ai/overview/privacy","reviewed_at":"2026-11-14"}
        },"metadata":model})
}
pub fn views(catalog: &Value, selected: &str) -> Vec<Value> {
    let mut views = Vec::new();
    for (key, historical) in [("models", false), ("historical_models", true)] {
        if let Some(models) = catalog[key].as_array() {
            for model in models {
                let availability = if historical {
                    "historical"
                } else {
                    match model["model_spec"]["offline"].as_bool() {
                        Some(false) => "available",
                        Some(true) => "offline",
                        None => "unknown",
                    }
                };
                let (model, fetched) = if historical {
                    (model.get("model").unwrap_or(model), &model["fetched_at"])
                } else {
                    (model, &catalog["fetched_at"])
                };
                views.push(describe(model, availability, fetched));
            }
        }
    }
    if !views.iter().any(|m| m["id"] == selected) {
        views.push(describe(
            &json!({"id":selected}),
            "historical",
            &Value::Null,
        ))
    }
    views
}
