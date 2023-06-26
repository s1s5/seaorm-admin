use super::{json_overwrite_key, templates, Admin, Json, ModelAdminTrait};
use askama::Template;
use rocket::request::Request;
use rocket::response::Responder;
use rocket::{
    get,
    http::Status,
    post,
    request::{FromRequest, Outcome},
    response::content,
    routes, Route, State,
};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub accept_html: bool,
    pub accept_json: bool,
    pub path: String,
    pub query: HashMap<String, Vec<String>>,
}

#[async_trait::async_trait]
impl<'r> FromRequest<'r> for RequestInfo {
    type Error = std::io::Error;
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let query = if let Some(query) = request.uri().query() {
            let mut m: HashMap<String, Vec<String>> = HashMap::new();
            for item in query.segments() {
                if m.contains_key(item.0) {
                    m.get_mut(item.0).unwrap().push(item.1.to_string());
                } else {
                    m.insert(item.0.to_string(), vec![item.1.to_string()]);
                }
            }
            m
        } else {
            HashMap::new()
        };

        let accept_html = request.headers().get("accept").find(|x| x.contains("html"));
        let accept_json = request.headers().get("accept").find(|x| x.contains("json"));
        Outcome::Success(RequestInfo {
            accept_html: accept_html.is_some(),
            accept_json: accept_json.is_some(),
            path: request.uri().path().as_str().to_string(),
            query: query,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HtmlOrJson {
    Json(rocket::response::content::RawJson<String>),
    Html(rocket::response::content::RawHtml<String>),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for HtmlOrJson {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'o> {
        match self {
            HtmlOrJson::Json(j) => j.respond_to(req),
            HtmlOrJson::Html(h) => h.respond_to(req),
        }
    }
}

fn return_json_object(
    model: &Box<dyn ModelAdminTrait + Send + Sync>,
    r: super::Result<Json>,
) -> (Status, content::RawJson<String>) {
    match r {
        Ok(data) => (
            Status::Ok,
            content::RawJson::<String>(
                serde_json::to_string(&json!({
                    "status": "ok",
                    // TODO: ErrのときInternalServerErrorにしないと、、
                    "key": serde_json::Value::String(model.json_to_key(&data).unwrap()),
                    "label": serde_json::Value::String(model.to_str(&data).unwrap()),
                    "data": data,

                }))
                .unwrap(),
            ),
        ),
        Err(error) => (
            Status::InternalServerError,
            content::RawJson::<String>(
                serde_json::to_string(&json!({
                    "status": "failed",
                    "error": format!("{}", error)
                }))
                .unwrap(),
            ),
        ),
    }
}

fn return_json<T>(r: super::Result<T>) -> (Status, content::RawJson<String>) {
    match r {
        Ok(_) => (
            Status::Ok,
            content::RawJson::<String>(
                serde_json::to_string(&json!({
                    "status": "ok"
                }))
                .unwrap(),
            ),
        ),
        Err(error) => (
            Status::InternalServerError,
            content::RawJson::<String>(
                serde_json::to_string(&json!({
                    "status": "failed",
                    "error": format!("{}", error)
                }))
                .unwrap(),
            ),
        ),
    }
}

#[get("/")]
pub async fn index(admin: &State<Admin>) -> Result<content::RawHtml<String>, Status> {
    let template =
        templates::AdminIndex::new(&admin.site).map_err(|_x| Status::InternalServerError)?;
    Ok(content::RawHtml(template.render().unwrap()))
}

#[get("/<model>?<_query..>")]
pub async fn list(
    model: &str,
    admin: &State<Admin>,
    _query: Option<String>,
    request_info: RequestInfo,
) -> Result<HtmlOrJson, Status> {
    let model = admin.models.get(model).ok_or(Status::NotFound)?;
    if (!request_info.accept_html) && request_info.accept_json {
        let object_list = admin
            .get_list_as_json(model, &request_info.query)
            .await
            .map_err(|_x| Status::InternalServerError)?;
        Ok(HtmlOrJson::Json(content::RawJson(
            serde_json::to_string(&object_list).map_err(|_x| Status::InternalServerError)?,
        )))
    } else {
        let template = admin
            .get_list_template(model, &request_info.query)
            .await
            .map_err(|_x| Status::InternalServerError)?;

        Ok(HtmlOrJson::Html(content::RawHtml(
            template.render().unwrap(),
        )))
    }
}

#[get("/<model>/create")]
pub async fn get_create_template(
    model: &str,
    admin: &State<Admin>,
) -> Result<content::RawHtml<String>, Status> {
    let model = admin.models.get(model).ok_or(Status::NotFound)?;
    let template = admin
        .get_create_template(model)
        .await
        .map_err(|_x| Status::InternalServerError)?;
    Ok(content::RawHtml(template.render().unwrap()))
}

#[post("/<model>/create", data = "<data>")]
pub async fn create_model<'r>(
    model: &str,
    data: &[u8],
    admin: &State<Admin>,
) -> Result<(Status, content::RawJson<String>), Status> {
    let data: serde_json::Value = serde_json::from_slice(data).unwrap();
    let model = admin.models.get(model).ok_or(Status::NotFound)?;
    Ok(return_json_object(
        model,
        model.insert(&admin.get_connection(), data).await,
    ))
}

#[get("/<model>/update/<id>")]
pub async fn get_update_template(
    model: &str,
    id: &str,
    admin: &State<Admin>,
) -> Result<content::RawHtml<String>, Status> {
    let model = admin.models.get(model).ok_or(Status::NotFound)?;
    let key = model.key_to_json(id).map_err(|_x| Status::BadRequest)?;
    let row = model
        .get(&admin.get_connection(), key)
        .await
        .map_err(|_x| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let template = admin
        .get_update_template(model, &row)
        .await
        .map_err(|_x| Status::InternalServerError)?;

    Ok(content::RawHtml(template.render().unwrap()))
}

#[post("/<model>/update/<id>", data = "<data>")]
pub async fn update_model(
    model: &str,
    id: &str,
    data: &[u8],
    admin: &State<Admin>,
) -> Result<(Status, content::RawJson<String>), Status> {
    let model = admin.models.get(model).ok_or(Status::NotFound)?;
    let key = model.key_to_json(id).map_err(|_x| Status::BadRequest)?;
    let data: serde_json::Value = serde_json::from_slice(data).unwrap();
    let data = json_overwrite_key(&data, &key).map_err(|_x| Status::InternalServerError)?;
    Ok(return_json_object(
        model,
        model.update(&admin.get_connection(), data).await,
    ))
}

#[get("/<model>/delete/<id>")]
pub async fn get_delete_template(
    model: &str,
    id: &str,
    admin: &State<Admin>,
) -> Result<content::RawHtml<String>, Status> {
    let model = admin.models.get(model).ok_or(Status::NotFound)?;
    let key = model.key_to_json(id).map_err(|_x| Status::BadRequest)?;
    let row = model
        .get(&admin.get_connection(), key)
        .await
        .map_err(|_x| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let template = admin
        .get_delete_template(model, &row)
        .await
        .map_err(|_x| Status::InternalServerError)?;

    Ok(content::RawHtml(template.render().unwrap()))
}

#[post("/<model>/delete/<id>", data = "<data>")]
pub async fn delete_model(
    model: &str,
    id: &str,
    data: &[u8],
    admin: &State<Admin>,
) -> Result<(Status, content::RawJson<String>), Status> {
    let model = admin.models.get(model).ok_or(Status::NotFound)?;
    let key = model.key_to_json(id).map_err(|_x| Status::BadRequest)?;
    let data: serde_json::Value = serde_json::from_slice(data).unwrap();
    let data = json_overwrite_key(&data, &key).map_err(|_x| Status::InternalServerError)?;

    Ok(return_json(
        model.delete(&admin.get_connection(), data).await,
    ))
}

pub fn get_admin_routes() -> Vec<Route> {
    routes![
        index,
        list,
        get_create_template,
        create_model,
        get_update_template,
        update_model,
        get_delete_template,
        delete_model,
    ]
}
