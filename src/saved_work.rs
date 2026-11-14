//! Read-only views of the private attempt records. Sidecars remain the source of truth.
use crate::{ApiResult, App, bad, error};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Default, Deserialize)]
pub struct Search {
    #[serde(default)]
    search: String,
    #[serde(default)]
    order: String,
}

fn fields(source: &Value, names: &[&str]) -> Value {
    Value::Object(
        names
            .iter()
            .filter_map(|name| {
                source
                    .get(name)
                    .map(|value| ((*name).into(), value.clone()))
            })
            .collect(),
    )
}

pub async fn export_recipe(State(app): State<App>, Path(id): Path<Uuid>) -> Response {
    let memory = app.state.lock().unwrap();
    let Some(a) = memory.attempts.get(&id) else {
        return error(StatusCode::NOT_FOUND, "Attempt not found.").into_response();
    };
    // Export authored choices and selected evidence, never the diagnostic capture.
    // New provider response fields cannot silently enter this download.
    let request = fields(
        &a.request,
        &[
            "model",
            "prompt",
            "safe_mode",
            "enhance_prompt",
            "return_binary",
            "embed_exif_metadata",
            "variants",
            "width",
            "height",
            "aspect_ratio",
            "resolution",
            "quality",
            "format",
            "seed",
            "negative_prompt",
            "steps",
            "cfg_scale",
            "style_preset",
            "transparent_background",
            "enable_web_search",
            "disable_prompt_optimization_thinking",
        ],
    );
    let recipe = app.provider.redact(&json!({
        "export_version":1,"recipe_version":a.recipe_version,"app_version":a.app_version,
        "id":a.id,"parent_id":a.parent_id,"created_at":a.created_at,
        "dispatched_at":a.dispatched_at,"completed_at":a.completed_at,
        "prompt":a.prompt,"request":request,"omitted_fields":a.omitted_fields,
        "model":{"id":a.request["model"],"metadata":fields(&a.model_snapshot["model_spec"], &[
            "name","constraints","privacy","uncensored","pricing"
        ])},
        "control_contract":a.control_contract,"catalog_source":a.catalog_source,
        "catalog_fetched_at":a.catalog_fetched_at,"estimate":a.estimate,
        "requested_seed":a.requested_seed,"returned_seed":a.returned_seed,
        "status":a.status,"outcome":a.outcome,"billing":fields(&a.billing, &["charge","refund"]),
        "results":a.results,
        "export_note":"Contains your private prompt and settings. Provider diagnostic bodies and explanations are excluded. Original images are downloaded separately. Estimates are not charges; repeating a recipe does not guarantee identical output."
    }));
    (
        [
            (header::CONTENT_TYPE, "application/json".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{id}.recipe.json\""),
            ),
        ],
        serde_json::to_vec_pretty(&recipe).unwrap(),
    )
        .into_response()
}

pub async fn search(State(app): State<App>, Query(query): Query<Search>) -> ApiResult<Value> {
    if !["", "newest", "oldest"].contains(&query.order.as_str()) {
        return Err(bad("Choose newest or oldest date order."));
    }
    let needle = query.search.trim().to_lowercase();
    let memory = app.state.lock().unwrap();
    let mut matches: Vec<_> = memory
        .attempts
        .values()
        .filter(|a| {
            [
                &a.prompt,
                a.request["model"].as_str().unwrap_or(""),
                a.model_snapshot["model_spec"]["name"]
                    .as_str()
                    .unwrap_or(""),
            ]
            .iter()
            .any(|value| value.to_lowercase().contains(&needle))
        })
        .collect();
    matches.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
    if query.order != "oldest" {
        matches.reverse();
    }
    Ok(Json(
        json!({"total":memory.attempts.len(),"attempts":matches.iter().map(|a| json!({
        "id":a.id,"created_at":a.created_at,"prompt":a.prompt,
        "model":a.request["model"],"model_name":a.model_snapshot["model_spec"]["name"],
        "status":a.status,"outcome":a.outcome,"summary":a.summary,"result_count":a.results.len()
    })).collect::<Vec<_>>()}),
    ))
}
