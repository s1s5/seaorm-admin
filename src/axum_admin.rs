use crate::create_cond_from_json;

use super::{json_overwrite_key, templates, Admin, ModelAdminTrait};
use askama::Template;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router, TypedHeader,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

// ----- HtmlOrJson -----
#[derive(Debug, Clone)]
enum HtmlOrJson {
    Json(Json<AnyData>),
    Html(Html<String>),
}

impl IntoResponse for HtmlOrJson {
    fn into_response(self) -> Response {
        match self {
            HtmlOrJson::Json(j) => j.into_response(),
            HtmlOrJson::Html(h) => h.into_response(),
        }
    }
}

// ----- RequestInfo -----
#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub accept_html: bool,
    pub accept_json: bool,
    pub path: String,
    pub query: HashMap<String, Vec<String>>,
}

// ----- AnyData -----
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AnyData(serde_json::Value);

// ----- Accept -----
enum RequestHeaderAccept {
    Json,
    Html,
}

impl axum::headers::Header for RequestHeaderAccept {
    fn name() -> &'static axum::headers::HeaderName {
        &axum::http::header::ACCEPT
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let value = values.next().ok_or_else(axum::headers::Error::invalid)?;
        let value = value
            .to_str()
            .map_err(|_| axum::headers::Error::invalid())?;

        if value.contains("json") {
            Ok(RequestHeaderAccept::Json)
        } else {
            Ok(RequestHeaderAccept::Html)
        }
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, _values: &mut E) {
        // never
    }
}

// ----- return json -----
fn return_json_object(
    model: &Box<dyn ModelAdminTrait + Send + Sync>,
    r: super::Result<super::Json>,
) -> (StatusCode, Json<AnyData>) {
    match r {
        Ok(data) => (
            StatusCode::OK,
            Json(AnyData(serde_json::json!({
                "status": "ok",
                // TODO: ErrのときInternalServerErrorにしないと、、
                "key": serde_json::Value::String(model.json_to_key(&data).unwrap()),
                "label": serde_json::Value::String(model.to_str(&data).unwrap()),
                "data": data,
            }))),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AnyData(serde_json::json!({
                "status": "failed",
                "error": format!("{}", error)
            }))),
        ),
    }
}
fn return_json<T>(r: super::Result<T>) -> (StatusCode, Json<AnyData>) {
    match r {
        Ok(_) => (
            StatusCode::OK,
            Json(AnyData(serde_json::json!({
                "status": "ok"
            }))),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AnyData(serde_json::json!({
                "status": "failed",
                "error": format!("{}", error)
            }))),
        ),
    }
}

// ----- routes -----
async fn index(Extension(admin): Extension<Arc<Admin>>) -> Result<Html<String>, StatusCode> {
    let template =
        templates::AdminIndex::new(&admin.site).map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(template.render().unwrap()))
}

async fn list(
    Path(model): Path<String>,
    Extension(admin): Extension<Arc<Admin>>,
    TypedHeader(accept): TypedHeader<RequestHeaderAccept>,
    Query(query): Query<HashMap<String, String>>, // TODO: array not supported
) -> Result<HtmlOrJson, StatusCode> {
    let request_info = RequestInfo {
        accept_html: true,
        accept_json: false,
        path: "/".to_string(),
        query: query.into_iter().map(|(k, v)| (k, vec![v])).collect(),
    };
    let model = admin.models.get(&model).ok_or(StatusCode::NOT_FOUND)?;
    match accept {
        RequestHeaderAccept::Json => {
            let object_list = admin
                .get_list_as_json(model, &request_info.query)
                .await
                .map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(HtmlOrJson::Json(Json(AnyData(object_list))))
        }
        RequestHeaderAccept::Html => {
            let template = admin
                .get_list_template(model, &request_info.query)
                .await
                .map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(HtmlOrJson::Html(Html(template.render().unwrap())))
        }
    }
}

async fn get_create_template(
    Path(model): Path<String>,
    Extension(admin): Extension<Arc<Admin>>,
) -> Result<Html<String>, StatusCode> {
    let model = admin.models.get(&model).ok_or(StatusCode::NOT_FOUND)?;
    let template = admin
        .get_create_template(model)
        .await
        .map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(template.render().unwrap()))
}

async fn create_model<'r>(
    Path(model): Path<String>,
    Extension(admin): Extension<Arc<Admin>>,
    Json(data): Json<AnyData>,
) -> Result<(StatusCode, Json<AnyData>), StatusCode> {
    let model = admin.models.get(&model).ok_or(StatusCode::NOT_FOUND)?;
    Ok(return_json_object(
        model,
        admin.create(model, &data.0).await,
    ))
}

async fn get_update_template(
    Path((model, id)): Path<(String, String)>,
    Extension(admin): Extension<Arc<Admin>>,
) -> Result<Html<String>, StatusCode> {
    let model = admin.models.get(&model).ok_or(StatusCode::NOT_FOUND)?;
    let key = model
        .key_to_json(&id)
        .map_err(|_x| StatusCode::BAD_REQUEST)?;
    let cond = create_cond_from_json(&model.get_primary_keys(), &key, true)
        .map_err(|_x| StatusCode::BAD_REQUEST)?;
    let row = model
        .get(&admin.get_connection(), &cond)
        .await
        .map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let template = admin
        .get_update_template(model, &row)
        .await
        .map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(template.render().unwrap()))
}

async fn update_model(
    Path((model, id)): Path<(String, String)>,
    Extension(admin): Extension<Arc<Admin>>,
    Json(data): Json<AnyData>,
) -> Result<(StatusCode, Json<AnyData>), StatusCode> {
    let model = admin.models.get(&model).ok_or(StatusCode::NOT_FOUND)?;
    let key = model
        .key_to_json(&id)
        .map_err(|_x| StatusCode::BAD_REQUEST)?;
    let data = json_overwrite_key(&data.0, &key).map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(return_json_object(model, admin.update(model, &data).await))
}

async fn get_delete_template(
    Path((model, id)): Path<(String, String)>,
    Extension(admin): Extension<Arc<Admin>>,
) -> Result<Html<String>, StatusCode> {
    let model = admin.models.get(&model).ok_or(StatusCode::NOT_FOUND)?;
    let key = model
        .key_to_json(&id)
        .map_err(|_x| StatusCode::BAD_REQUEST)?;
    let cond = create_cond_from_json(&model.get_primary_keys(), &key, true)
        .map_err(|_x| StatusCode::BAD_REQUEST)?;
    let row = model
        .get(&admin.get_connection(), &cond)
        .await
        .map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let template = admin
        .get_delete_template(model, &row)
        .await
        .map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(template.render().unwrap()))
}

async fn delete_model(
    Path((model, id)): Path<(String, String)>,
    Extension(admin): Extension<Arc<Admin>>,
    Json(data): Json<AnyData>,
) -> Result<(StatusCode, Json<AnyData>), StatusCode> {
    let model = admin.models.get(&model).ok_or(StatusCode::NOT_FOUND)?;
    let key = model
        .key_to_json(&id)
        .map_err(|_x| StatusCode::BAD_REQUEST)?;

    let data = json_overwrite_key(&data.0, &key).map_err(|_x| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(return_json(admin.delete(model, &data).await))
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/:model/", get(list))
        .route(
            "/:model/create/",
            get(get_create_template).post(create_model),
        )
        .route(
            "/:model/update/:id/",
            get(get_update_template).post(update_model),
        )
        .route(
            "/:model/delete/:id/",
            get(get_delete_template).post(delete_model),
        )
}
