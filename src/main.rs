mod library;
mod provider;
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use base64::Engine;
use chrono::Utc;
use library::Library;
use provider::Provider;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

type App = Arc<Studio>;
type ApiResult<T> = Result<Json<T>, (StatusCode, Json<Value>)>;
fn error(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"error":message.into()})))
}
fn bad(message: impl Into<String>) -> (StatusCode, Json<Value>) {
    error(StatusCode::BAD_REQUEST, message)
}
fn now() -> String {
    Utc::now().to_rfc3339()
}
#[derive(Clone, Serialize, Deserialize)]
struct Draft {
    id: Uuid,
    prompt: String,
    safe_mode: bool,
    model: String,
    #[serde(default)]
    parent_id: Option<Uuid>,
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            prompt: String::new(),
            safe_mode: false,
            model: "chroma".into(),
            parent_id: None,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct ResultImage {
    id: Uuid,
    width: u32,
    height: u32,
    mime: String,
    bytes: usize,
    sha256: String,
    relative_path: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Attempt {
    recipe_version: u32,
    app_version: String,
    id: Uuid,
    created_at: String,
    dispatched_at: Option<String>,
    completed_at: Option<String>,
    parent_id: Option<Uuid>,
    prompt: String,
    request: Value,
    omitted_fields: Vec<String>,
    model_snapshot: Value,
    catalog_source: Value,
    catalog_fetched_at: Value,
    estimate: Value,
    requested_seed: Option<i64>,
    returned_seed: Value,
    status: String,
    summary: String,
    outcome: String,
    outcome_summary: String,
    billing: Value,
    provider: Value,
    results: Vec<ResultImage>,
}
struct Memory {
    draft: Draft,
    catalog: Value,
    catalog_error: Option<String>,
    attempts: BTreeMap<Uuid, Attempt>,
    inflight: Option<Uuid>,
    bytes: BTreeMap<Uuid, Vec<u8>>,
    setup: Option<String>,
}
struct Studio {
    library: Option<Library>,
    provider: Provider,
    state: Mutex<Memory>,
    token: String,
    origin: String,
}
#[tokio::main]
async fn main() {
    if let Err(message) = run().await {
        eprintln!("Studio setup: {message}");
        std::process::exit(1)
    }
}
async fn run() -> Result<(), String> {
    let mut port = 4318;
    let mut fixture = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" => {
                port = args
                    .next()
                    .ok_or("Missing port.")?
                    .parse::<u16>()
                    .map_err(|_| "Invalid port.")?
            }
            "--fixture-provider" => fixture = Some(args.next().ok_or("Missing fixture origin.")?),
            _ => return Err("Unknown option. Use --port PORT.".into()),
        }
    }
    // Fixture mode never reads private .env or consumes a process credential.
    if fixture.is_none() {
        let _ = dotenvy::from_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env"));
    }
    let key = if fixture.is_none() {
        std::env::var("VENICE_API_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty())
    } else {
        None
    };
    let provider = Provider::new(key, fixture)?;
    let path = std::env::var("VENICE_IMAGE_STUDIO_DATA_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| PathBuf::from(home).join("Pictures/Venice Image Studio"))
        })
        .ok_or("Set HOME or VENICE_IMAGE_STUDIO_DATA_DIR.")?;
    let (library, setup) = match Library::open(path) {
        Ok(l) => (Some(l), None),
        Err(e) => (None, Some(e)),
    };
    let mut memory = Memory {
        draft: Draft::default(),
        catalog: Value::Null,
        catalog_error: None,
        attempts: BTreeMap::new(),
        inflight: None,
        bytes: BTreeMap::new(),
        setup,
    };
    if let Some(l) = &library {
        if let Ok(bytes) = l.read("draft.json", 262144)
            && let Ok(d) = serde_json::from_slice(&bytes)
        {
            memory.draft = d;
        }
        if let Ok(bytes) = l.read("catalog.json", 4 * 1024 * 1024)
            && let Ok(c) = serde_json::from_slice(&bytes)
        {
            memory.catalog = c;
        }
        for name in l.recipes()? {
            let bytes = l.read(&name, 512 * 1024)?;
            let mut a:Attempt=serde_json::from_slice(&bytes).map_err(|_|"A library recipe cannot be read. Preserve the library and repair that record before starting.")?;
            if ["dispatching", "waiting"].contains(&a.status.as_str()) {
                a.status = "indeterminate".into();
                a.outcome = "indeterminate".into();
                a.summary="Studio stopped after dispatch. Provider outcome and billing are unknown. Nothing was retried.".into();
                save_recipe(l, &a)?;
            }
            if a.status == "saving" || a.status == "save_failed" {
                let all = !a.results.is_empty()
                    && a.results.iter().all(|r| {
                        l.read(&r.relative_path, 32 * 1024 * 1024)
                            .is_ok_and(|b| hash(&b) == r.sha256)
                    });
                if all {
                    a.status = "saved".into();
                    a.summary = "Recovered durable originals after an interrupted save.".into();
                    save_recipe(l, &a)?;
                } else {
                    a.status = "save_failed".into();
                    a.summary="Save was interrupted. Only originals still available locally can be recovered; nothing was regenerated.".into();
                }
            }
            memory.attempts.insert(a.id, a);
        }
    }
    let origin = format!("http://127.0.0.1:{port}");
    let app = Arc::new(Studio {
        library,
        provider,
        state: Mutex::new(memory),
        token: Uuid::new_v4().to_string(),
        origin: origin.clone(),
    });
    refresh(&app).await;
    let router = Router::new()
        .route("/", get(index))
        .route("/app.js", get(js))
        .route("/style.css", get(css))
        .route("/api/state", get(state))
        .route("/api/draft", put(draft))
        .route("/api/catalog/refresh", post(refresh_route))
        .route("/api/generate", post(generate))
        .route("/api/attempts/{id}", get(attempt))
        .route("/api/attempts/{id}/reuse", post(reuse))
        .route("/api/attempts/{id}/save", post(retry_save))
        .route("/api/images/{id}", get(original))
        .layer(DefaultBodyLimit::max(262144))
        .layer(middleware::from_fn_with_state(app.clone(), guard))
        .with_state(app);
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        .await
        .map_err(|_| "Cannot bind loopback port; another Studio may be running.")?;
    println!("Venice Image Studio: {origin}");
    axum::serve(listener, router)
        .await
        .map_err(|_| "Local server stopped unexpectedly.".into())
}
async fn guard(State(app): State<App>, req: Request, next: Next) -> Response {
    let expected = app.origin.trim_start_matches("http://");
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|x| x.to_str().ok());
    let origin = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|x| x.to_str().ok());
    let cross = req
        .headers()
        .get("sec-fetch-site")
        .and_then(|x| x.to_str().ok())
        == Some("cross-site");
    let mutation = !matches!(req.method().as_str(), "GET" | "HEAD");
    if host != Some(expected)
        || origin.is_some_and(|v| v != app.origin)
        || cross
        || (mutation
            && (origin != Some(app.origin.as_str())
                || req
                    .headers()
                    .get("x-studio-token")
                    .and_then(|x| x.to_str().ok())
                    != Some(app.token.as_str())))
    {
        return error(
            StatusCode::FORBIDDEN,
            "Request must come from this Studio window.",
        )
        .into_response();
    }
    let mut response = next.run(req).await;
    for (k, v) in [
        (
            "content-security-policy",
            "default-src 'self'; img-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
        ),
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        ("cache-control", "no-store"),
    ] {
        response.headers_mut().insert(
            header::HeaderName::from_static(k),
            header::HeaderValue::from_static(v),
        );
    }
    response
}
async fn index() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("../web/index.html"),
    )
}
async fn js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../web/dist/app.js"),
    )
}
async fn css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../web/style.css"),
    )
}
async fn state(State(app): State<App>) -> Json<Value> {
    let m = app.state.lock().unwrap();
    Json(
        json!({"token":app.token,"draft":m.draft,"catalog":m.catalog,"catalog_error":m.catalog_error,"attempts":m.attempts.values().rev().collect::<Vec<_>>(),"inflight":m.inflight,"setup_error":m.setup,"credential_present":app.provider.key.is_some(),"library":app.library.as_ref().map(|l|l.root.display().to_string())}),
    )
}
async fn draft(State(app): State<App>, Json(d): Json<Draft>) -> ApiResult<Draft> {
    if d.prompt.chars().count() > 32768 {
        return Err(bad(
            "Draft exceeds the local 32,768-character storage bound.",
        ));
    }
    let l = app
        .library
        .as_ref()
        .ok_or_else(|| bad("Resolve the library setup problem before saving drafts."))?;
    let mut m = app.state.lock().unwrap();
    if m.attempts.contains_key(&d.id) {
        return Err(bad(
            "This draft was already submitted. Create a new draft to make another explicit attempt.",
        ));
    }
    l.write("draft.json", &serde_json::to_vec(&d).unwrap())
        .map_err(bad)?;
    m.draft = d.clone();
    Ok(Json(d))
}
async fn refresh(app: &App) {
    match app.provider.catalog().await {
        Ok(c) => {
            let result = app
                .library
                .as_ref()
                .map(|l| l.write("catalog.json", &serde_json::to_vec(&c).unwrap()));
            let mut m = app.state.lock().unwrap();
            m.catalog = c;
            m.catalog_error = result.and_then(Result::err);
        }
        Err(e) => app.state.lock().unwrap().catalog_error = Some(e),
    }
}
async fn refresh_route(State(app): State<App>) -> Json<Value> {
    refresh(&app).await;
    state(State(app)).await
}
#[derive(Deserialize)]
struct Generate {
    draft_id: Uuid,
    catalog_fetched_at: String,
}
async fn generate(State(app): State<App>, Json(g): Json<Generate>) -> ApiResult<Attempt> {
    let a = {
        let mut m = app.state.lock().unwrap();
        if let Some(a) = m.attempts.get(&g.draft_id) {
            return Ok(Json(a.clone()));
        }
        if m.catalog["fetched_at"].as_str() != Some(g.catalog_fetched_at.as_str()) {
            return Err(error(
                StatusCode::CONFLICT,
                "Catalog changed. Inspect the refreshed estimate and settings before generating.",
            ));
        }
        if m.inflight.is_some() {
            return Err(error(
                StatusCode::CONFLICT,
                "An attempt is already waiting for Venice.",
            ));
        }
        if app.provider.key.is_none() {
            return Err(bad(
                "Set VENICE_API_KEY in the private .env or process environment and restart Studio.",
            ));
        }
        let l = app
            .library
            .as_ref()
            .ok_or_else(|| bad("Resolve the library setup problem before generation."))?;
        l.preflight().map_err(bad)?;
        let d = &m.draft;
        if d.id != g.draft_id {
            return Err(bad(
                "Draft changed. Reload and inspect it before generation.",
            ));
        }
        if d.model != "chroma" {
            return Err(bad(
                "This first slice supports the selected Chroma generation path.",
            ));
        }
        let model = m.catalog["models"]
            .as_array()
            .and_then(|a| a.iter().find(|v| v["id"] == d.model))
            .ok_or_else(|| bad("Refresh the catalog; selected model is unavailable."))?
            .clone();
        if model["model_spec"]["offline"] != false {
            return Err(bad("Selected model availability is unknown or offline."));
        }
        if serde_json::to_vec(&model).unwrap().len() > 65536 {
            return Err(bad(
                "Selected model metadata exceeds the local capture bound.",
            ));
        }
        let spec = &model["model_spec"];
        let limit = spec["constraints"]["promptCharacterLimit"]
            .as_u64()
            .ok_or_else(|| bad("Selected model's prompt limit is unknown."))?;
        if d.prompt.is_empty() || d.prompt.chars().count() as u64 > limit {
            return Err(bad(format!(
                "Prompt must contain 1–{limit} Unicode scalar characters."
            )));
        }
        let divisor = spec["constraints"]["widthHeightDivisor"]
            .as_u64()
            .filter(|n| *n > 0)
            .ok_or_else(|| bad("Selected model's size constraint is unknown."))?;
        if 1024 % divisor != 0 {
            return Err(bad(
                "Refreshed model no longer supports the selected 1024-pixel size.",
            ));
        }
        let price = spec["pricing"]["generation"]["usd"]
            .as_f64()
            .filter(|p| p.is_finite() && *p >= 0.0)
            .ok_or_else(|| {
                bad("No usable estimate is available; refresh the catalog before spending.")
            })?;
        if !spec["pricing"]["resolutions"].is_null() || !spec["pricing"]["quality"].is_null() {
            return Err(bad(
                "The selected model's pricing contract changed; this request needs review.",
            ));
        }
        let request = json!({"model":d.model,"prompt":d.prompt,"width":1024,"height":1024,"variants":1,"safe_mode":d.safe_mode,"enhance_prompt":false,"return_binary":false,"embed_exif_metadata":false,"format":"png"});
        let a = Attempt {
            recipe_version: 1,
            app_version: env!("CARGO_PKG_VERSION").into(),
            id: d.id,
            created_at: now(),
            dispatched_at: Some(now()),
            completed_at: None,
            parent_id: d.parent_id,
            prompt: d.prompt.clone(),
            request,
            omitted_fields: [
                "seed",
                "negative_prompt",
                "steps",
                "cfg_scale",
                "style_preset",
                "enable_web_search",
                "disable_prompt_optimization_thinking",
            ]
            .map(String::from)
            .to_vec(),
            model_snapshot: model,
            catalog_source: m.catalog["source"].clone(),
            catalog_fetched_at: m.catalog["fetched_at"].clone(),
            estimate: json!({"usd":price,"source":m.catalog["source"],"fetched_at":m.catalog["fetched_at"],"kind":"estimate, not charge or cap"}),
            requested_seed: None,
            returned_seed: Value::Null,
            outcome: "pending".into(),
            outcome_summary: "Waiting for Venice.".into(),
            status: "waiting".into(),
            summary: "Waiting for Venice. This request continues if the browser reloads.".into(),
            billing: json!({"charge":"unknown","refund":"unknown"}),
            provider: Value::Null,
            results: vec![],
        };
        save_recipe(l, &a).map_err(bad)?;
        m.inflight = Some(a.id);
        m.attempts.insert(a.id, a.clone());
        a
    };
    let task = app.clone();
    let id = a.id;
    tokio::spawn(async move {
        dispatch(task, id).await;
    });
    Ok(Json(a))
}
async fn dispatch(app: App, id: Uuid) {
    let request = app.state.lock().unwrap().attempts[&id].request.clone();
    let response = app.provider.generate(&request).await;
    let mut m = app.state.lock().unwrap();
    let mut a = m.attempts[&id].clone();
    a.completed_at = Some(now());
    match response {
        Err(e) => {
            a.status = "indeterminate".into();
            a.summary = e;
        }
        Ok((status, body, headers)) => {
            let details = app
                .provider
                .redact(&json!({"http_status":status,"body":body,"headers":headers}));
            let encoded = serde_json::to_string(&details).unwrap();
            a.provider = if encoded.len() > 32768 {
                json!({"http_status":status,"capture_truncated":true,"text":encoded.chars().take(16000).collect::<String>()})
            } else {
                details
            };
            a.returned_seed = body
                .get("seed")
                .and_then(Value::as_i64)
                .map(Value::from)
                .unwrap_or(Value::Null);
            let mut invalid = 0;
            let mut decode_errors = Vec::new();
            if let Some(images) = body["images"].as_array() {
                for (index, value) in images.iter().enumerate() {
                    match decode(value) {
                        Ok((r, b)) => {
                            m.bytes.insert(r.id, b);
                            a.results.push(r)
                        }
                        Err(message) => {
                            invalid += 1;
                            if decode_errors.len() < 16 {
                                decode_errors.push(json!({"index":index,"error":message}));
                            }
                        }
                    }
                }
            }
            if invalid > 0 {
                a.provider["local_decode_errors"] = json!(decode_errors);
                a.provider["local_decode_errors_truncated"] = json!(invalid > 16);
            }
            if !a.results.is_empty() {
                a.status = "saving".into();
                a.outcome = if invalid > 0 { "partial" } else { "received" }.into();
                a.summary = if invalid > 0 {
                    format!(
                        "Received {} valid originals; {invalid} items could not be retained as valid images.",
                        a.results.len()
                    )
                } else {
                    "Received originals; saving locally.".into()
                };
            } else {
                a.status = if body["body_incomplete"] == true {
                    "indeterminate"
                } else if status == 401 || status == 403 {
                    "access_error"
                } else if status == 429 {
                    "rate_limited"
                } else if status == 422 {
                    "rejected"
                } else {
                    "failed"
                }
                .into();
                a.summary = if body["body_incomplete"] == true {
                    "Provider body incomplete or exceeded capture bounds. Outcome and billing are unknown; no retry was sent.".into()
                } else {
                    match status{401|403=>"Venice rejected access. Check the private API key and account permissions, then restart Studio.",422=>"Venice rejected the request. Inspect provider details; a policy reason is not assumed.",429=>"Venice rate limited the request. No retry was sent.",_=>"No valid image was received. Inspect provider details. Billing and refund are unknown."}.into()
                };
            }
        }
    }
    if a.status != "saving" {
        a.outcome = a.status.clone();
    }
    a.outcome_summary = a.summary.clone();
    m.attempts.insert(id, a);
    m.inflight = None;
    persist_attempt(&app, &mut m, id);
}
fn persist_attempt(app: &App, m: &mut Memory, id: Uuid) {
    let l = app.library.as_ref().unwrap();
    let mut a = m.attempts[&id].clone();
    let result = (|| {
        save_recipe(l, &a)?;
        for r in &a.results {
            if let Some(bytes) = m.bytes.get(&r.id) {
                l.write(&r.relative_path, bytes)?;
                // Test-only process interruption at the actual durable-write boundary.
                #[cfg(feature = "test-support")]
                if app.provider.is_fixture()
                    && std::env::var("VENICE_STUDIO_TEST_CRASH_AFTER_ORIGINAL").as_deref()
                        == Ok("1")
                {
                    std::process::exit(86);
                }
            } else {
                let bytes = l.read(&r.relative_path, 32 * 1024 * 1024)?;
                if hash(&bytes) != r.sha256 {
                    return Err("Original hash does not match the recipe.".into());
                }
            }
        }
        if !a.results.is_empty() {
            a.status = "saved".into();
            a.summary = if a.outcome == "partial" {
                format!("{} Originals and recipe saved locally.", a.outcome_summary)
            } else {
                "Originals and recipe saved locally.".into()
            };
        } else {
            a.status = a.outcome.clone();
            a.summary = a.outcome_summary.clone();
        }
        save_recipe(l, &a)
    })();
    if let Err(e) = result {
        a.status = "save_failed".into();
        a.summary = e;
        let _ = save_recipe(l, &a);
    } else {
        for r in &a.results {
            m.bytes.remove(&r.id);
        }
    }
    m.attempts.insert(id, a);
}
fn save_recipe(l: &Library, a: &Attempt) -> Result<(), String> {
    l.write(
        &format!("{}.recipe.json", a.id),
        &serde_json::to_vec_pretty(a).unwrap(),
    )
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn decode(value: &Value) -> Result<(ResultImage, Vec<u8>), String> {
    let text = value.as_str().ok_or("Image is not base64 text.")?;
    if text.len() > 44 * 1024 * 1024 {
        return Err("Image exceeds decode limit.".into());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|_| "Image is not valid base64.")?;
    if bytes.len() > 32 * 1024 * 1024 {
        return Err("Image exceeds original-byte limit.".into());
    }
    let format = image::guess_format(&bytes).map_err(|_| "Unknown image type.")?;
    let mime = match format {
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::WebP => "image/webp",
        _ => return Err("Unsupported image type.".into()),
    };
    let mut reader = image::ImageReader::with_format(std::io::Cursor::new(&bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader
        .decode()
        .map_err(|_| "Image decoding failed or exceeded limits.")?;
    let id = Uuid::new_v4();
    let r = ResultImage {
        id,
        width: decoded.width(),
        height: decoded.height(),
        mime: mime.into(),
        bytes: bytes.len(),
        sha256: hash(&bytes),
        relative_path: format!("{id}.original"),
    };
    Ok((r, bytes))
}
async fn attempt(State(app): State<App>, Path(id): Path<Uuid>) -> ApiResult<Attempt> {
    app.state
        .lock()
        .unwrap()
        .attempts
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Attempt not found."))
}
async fn reuse(State(app): State<App>, Path(id): Path<Uuid>) -> ApiResult<Draft> {
    let a = app
        .state
        .lock()
        .unwrap()
        .attempts
        .get(&id)
        .cloned()
        .ok_or_else(|| bad("Attempt not found."))?;
    draft(
        State(app),
        Json(Draft {
            id: Uuid::new_v4(),
            prompt: a.prompt,
            safe_mode: a.request["safe_mode"].as_bool().unwrap_or(false),
            model: a.request["model"].as_str().unwrap_or("chroma").into(),
            parent_id: Some(a.id),
        }),
    )
    .await
}
async fn retry_save(State(app): State<App>, Path(id): Path<Uuid>) -> ApiResult<Attempt> {
    let mut m = app.state.lock().unwrap();
    let a = m
        .attempts
        .get(&id)
        .ok_or_else(|| bad("Attempt not found."))?;
    if a.status != "save_failed" {
        return Err(bad("This attempt does not need a local save retry."));
    }
    persist_attempt(&app, &mut m, id);
    Ok(Json(m.attempts[&id].clone()))
}
async fn original(State(app): State<App>, Path(id): Path<Uuid>) -> Response {
    let m = app.state.lock().unwrap();
    let Some(r) = m
        .attempts
        .values()
        .flat_map(|a| a.results.iter())
        .find(|r| r.id == id)
    else {
        return error(StatusCode::NOT_FOUND, "Original not found.").into_response();
    };
    let bytes = m.bytes.get(&id).cloned().or_else(|| {
        app.library
            .as_ref()
            .and_then(|l| l.read(&r.relative_path, 32 * 1024 * 1024).ok())
    });
    match bytes {
        Some(bytes) if hash(&bytes) == r.sha256 => Response::builder()
            .header(header::CONTENT_TYPE, &r.mime)
            .header(
                header::CONTENT_DISPOSITION,
                format!(
                    "inline; filename=\"{}.{}\"",
                    id,
                    r.mime.trim_start_matches("image/")
                ),
            )
            .body(Body::from(bytes))
            .unwrap(),
        _ => error(
            StatusCode::NOT_FOUND,
            "Original unavailable or integrity check failed.",
        )
        .into_response(),
    }
}
