use crate::{ListQuery, Result};
pub use askama::{DynTemplate, Template};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AdminSite {
    pub title: String,
    pub sub_path: String,
    pub models: Vec<String>,
}

#[derive(Template, Clone)]
#[template(path = "input.jinja")]
pub struct AdminFormInput {
    pub name: String,
    pub label: String,
    pub r#type: String,
    pub value: Option<String>,
    pub help_text: Option<String>,
    pub disabled: bool,
    pub attributes: HashMap<String, String>,
}

#[derive(Template, Clone)]
#[template(path = "textarea.jinja")]
pub struct AdminFormTextarea {
    pub name: String,
    pub label: String,
    pub value: Option<String>,
    pub help_text: Option<String>,
    pub disabled: bool,
}

#[derive(Template)]
#[template(path = "checkbox.jinja")]
pub struct AdminFormCheckbox {
    pub name: String,
    pub label: String,
    pub checked: bool,
    pub help_text: Option<String>,
    pub disabled: bool,
}

#[derive(Template, Clone)]
#[template(path = "select.jinja")]
pub struct AdminFormSelect {
    pub name: String,
    pub label: String,
    pub value: String,
    pub help_text: Option<String>,
    pub disabled: bool,
    pub choices: Vec<(String, String)>,
    pub attributes: HashMap<String, String>,
}

pub struct AdminFormDatetimeInputValue {
    pub raw: String,
    pub datetime_without_seconds: String,
    pub seconds: f64,
    pub timezone: i32,
}

#[derive(Template)]
#[template(path = "datetime-input.jinja")]
pub struct AdminFormDatetimeInput {
    pub name: String,
    pub label: String,
    pub value: Option<AdminFormDatetimeInputValue>,
    pub with_timezone: bool,
    pub help_text: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct AdminFormAutoCompleteChoice {
    pub value: String,
    pub label: String,
    pub json_str: String,
}

#[derive(Debug, Clone)]
pub struct AdminFormAutoCompleteCol {
    pub value: Vec<String>,
    pub from_col: String,
    pub to_col: String,
}

#[derive(Template, Clone)]
#[template(path = "auto-complete.jinja")]
pub struct AdminFormAutoComplete {
    pub prefix: String,
    pub name: String,
    pub label: String,
    pub choices: Vec<AdminFormAutoCompleteChoice>,
    pub help_text: Option<String>,
    pub disabled: bool,
    pub to_table: String,
    pub cols: Vec<AdminFormAutoCompleteCol>,
    pub nullable: bool,
    pub multiple: bool,
}

pub struct RelationFormRowField {
    pub is_pkey: bool,
    pub field: Box<dyn DynTemplate + Send>,
}

pub struct RelationFormRow {
    pub is_update: bool,
    pub fields: Vec<RelationFormRowField>,
}

#[derive(Template)]
#[template(path = "relation-form.jinja")]
pub struct RelationForm {
    pub name: String,
    // pub nullable: bool,
    pub multiple: bool,
    pub template_fields: Vec<Box<dyn DynTemplate + Send>>,
    pub rows: Vec<RelationFormRow>,
}

#[derive(Template)]
#[template(path = "create-form.jinja")]
pub struct AdminCreateForm {
    pub site: AdminSite,
    pub form_id: String,
    pub page_id: String,
    pub model_name: String,
    pub action: Option<String>,
    pub method: String,
    pub fields: Vec<Box<dyn DynTemplate + Send>>,
}

#[derive(Template)]
#[template(path = "update-form.jinja")]
pub struct AdminUpdateForm {
    pub site: AdminSite,
    pub form_id: String,
    pub page_id: String,
    pub model_name: String,
    pub action: Option<String>,
    pub method: String,
    pub fields: Vec<Box<dyn DynTemplate + Send>>,
}

#[derive(Template)]
#[template(path = "delete-form.jinja")]
pub struct AdminDeleteForm {
    pub site: AdminSite,
    pub form_id: String,
    pub page_id: String,
    pub model_name: String,
    pub action: Option<String>,
    pub method: String,
    pub fields: Vec<Box<dyn DynTemplate + Send>>,
}

#[derive(Debug, Clone)]
pub struct AdminListPage {
    pub is_active: bool,
    pub link: Option<String>,
    pub label: String,
}

#[derive(Template)]
#[template(path = "list.jinja")]
pub struct AdminList {
    pub site: AdminSite,
    pub model_name: String,
    pub keys: Vec<String>,
    pub rows: Vec<(String, Vec<String>)>,
    pub query: ListQuery,
    pub pages: Vec<AdminListPage>,
    pub total: u64,
}

#[derive(Template)]
#[template(path = "index.jinja")]
pub struct AdminIndex {
    pub site: AdminSite,
}

impl AdminIndex {
    pub fn new(site: &AdminSite) -> Result<Self> {
        Ok(AdminIndex { site: site.clone() })
    }
}
