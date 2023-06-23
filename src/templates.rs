use super::{AdminField, CustomError, Json, Result};
use crate::{json_force_str, ListQuery};
pub use askama::{DynTemplate, Template};
#[cfg(feature = "with-chrono")]
use chrono::Timelike;
use log::warn;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AdminSite {
    pub title: String,
    pub sub_path: String,
    pub models: Vec<String>,
}

#[derive(Template)]
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

#[derive(Template)]
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

#[derive(Template)]
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
}
pub struct AdminFormAutoCompleteCol {
    pub from_col: String,
    pub to_col: String,
}

#[derive(Template)]
#[template(path = "auto-complete.jinja")]
pub struct AdminFormAutoComplete {
    pub name: String,
    pub label: String,
    pub choice: Option<AdminFormAutoCompleteChoice>,
    pub help_text: Option<String>,
    pub disabled: bool,
    pub to_table: String,
    pub cols: Vec<AdminFormAutoCompleteCol>,
    pub nullable: bool,
    pub multiple: bool,
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
    pub fields: Vec<Box<dyn DynTemplate>>,
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
    pub fields: Vec<Box<dyn DynTemplate>>,
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
    pub fields: Vec<Box<dyn DynTemplate>>,
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

pub fn create_form_field(field: &AdminField, value: Option<&Json>) -> Result<Box<dyn DynTemplate>> {
    let org_value = value;
    let value = value.map(|x| json_force_str(&x));
    let mut input = Box::new(AdminFormInput {
        name: field.name.clone(),
        label: field.name.clone(),
        r#type: "text".into(),
        value: value.clone(),
        help_text: None,
        disabled: !field.editable,
        attributes: HashMap::new(),
    });
    let textarea = Box::new(AdminFormTextarea {
        name: field.name.clone(),
        label: field.name.clone(),
        value: value.clone(),
        help_text: None,
        disabled: !field.editable,
    });

    Ok(match &field.column_type {
        sea_orm::ColumnType::Char(o) | sea_orm::ColumnType::String(o) => {
            if let Some(o) = o {
                input.attributes = HashMap::from_iter([("maxlength".to_string(), o.to_string())]);
                input
            } else {
                textarea
            }
        }
        sea_orm::ColumnType::Text | sea_orm::ColumnType::Json | sea_orm::ColumnType::JsonBinary => {
            textarea
        }
        sea_orm::ColumnType::TinyInteger
        | sea_orm::ColumnType::SmallInteger
        | sea_orm::ColumnType::Integer
        | sea_orm::ColumnType::BigInteger
        | sea_orm::ColumnType::TinyUnsigned
        | sea_orm::ColumnType::SmallUnsigned
        | sea_orm::ColumnType::Unsigned
        | sea_orm::ColumnType::BigUnsigned => {
            input.r#type = "number".into();
            input.attributes = HashMap::from_iter([("step".into(), "1".into())]);
            input
        }
        sea_orm::ColumnType::Float
        | sea_orm::ColumnType::Double
        | sea_orm::ColumnType::Money(_) => {
            input.r#type = "number".into();
            input.attributes = HashMap::from_iter([("step".into(), "auto".into())]);
            input
        }
        #[cfg(feature = "with-rust_decimal")]
        sea_orm::ColumnType::Decimal(_) => {
            input.r#type = "number".into();
            input.attributes = HashMap::from_iter([("step".into(), "auto".into())]);
            input
        }

        #[cfg(feature = "with-chrono")]
        sea_orm::ColumnType::DateTime | sea_orm::ColumnType::TimestampWithTimeZone => {
            let value = if let Some(value) = &value {
                if field.column_type == sea_orm::ColumnType::DateTime {
                    let v: Option<chrono::NaiveDateTime> =
                        serde_json::from_value(Json::String(value.clone())).ok();
                    if let Some(v) = v {
                        Some(AdminFormDatetimeInputValue {
                            raw: value.clone(),
                            datetime_without_seconds: v.format("%Y-%m-%dT%H:%M").to_string(),
                            seconds: v.time().second() as f64
                                + (v.timestamp_subsec_micros() as f64) * 1.0e-6,
                            timezone: 0,
                        })
                    } else {
                        None
                    }
                } else {
                    let v: Option<chrono::DateTime<chrono::FixedOffset>> =
                        serde_json::from_value(Json::String(value.clone())).ok();
                    if let Some(v) = v {
                        Some(AdminFormDatetimeInputValue {
                            raw: value.clone(),
                            datetime_without_seconds: v.format("%Y-%m-%dT%H:%M").to_string(),
                            seconds: v.time().second() as f64
                                + (v.timestamp_subsec_micros() as f64) * 1.0e-6,
                            timezone: v.timezone().local_minus_utc() * 60,
                        })
                    } else {
                        None
                    }
                }
            } else {
                None
            };
            Box::new(AdminFormDatetimeInput {
                name: field.name.clone(),
                label: field.name.clone(),
                value: value,
                with_timezone: field.column_type == sea_orm::ColumnType::TimestampWithTimeZone,
                help_text: None,
                disabled: !field.editable,
            })
        }

        sea_orm::ColumnType::Time => {
            input.r#type = "time".into();
            input
        }
        sea_orm::ColumnType::Date => {
            input.r#type = "date".into();
            input
        }
        sea_orm::ColumnType::Year(_o) => {
            input.r#type = "number".into();
            input
        }
        // sea_orm::ColumnType::Interval(o) => {}
        sea_orm::ColumnType::Binary(_) | sea_orm::ColumnType::VarBinary(_) => {
            // pattern="^[A-Za-z0-9+/]{4}*[A-Za-z0-9+/]{4}([A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$"
            textarea
            // input.attributes = HashMap::from_iter([
            //     ("pattern".into(), "[0-9a-fA-F]*".into()),
            //     ("title".into(), "input format must be hex string".into()),
            // ]);
            // input
        }
        // sea_orm::ColumnType::Bit(_o) => {
        //     input.attributes = HashMap::from_iter([
        //         ("pattern".into(), "[0-1]*".into()),
        //         ("title".into(), "input format must be bit string".into()),
        //     ]);
        //     input
        // }
        // sea_orm::ColumnType::VarBit(_o) => {
        //     input.attributes = HashMap::from_iter([
        //         ("pattern".into(), "[0-1]*".into()),
        //         ("title".into(), "input format must be bit string".into()),
        //     ]);
        //     input
        // }
        sea_orm::ColumnType::Boolean => Box::new(AdminFormCheckbox {
            name: field.name.clone(),
            label: field.name.clone(),
            checked: org_value
                .unwrap_or(&Json::Bool(false))
                .as_bool()
                .unwrap_or(false),
            help_text: None,
            disabled: !field.editable,
        }),
        //  => {}
        #[cfg(feature = "with-uuid")]
        sea_orm::ColumnType::Uuid => {
            input.attributes = HashMap::from_iter([
                (
                    "pattern".into(),
                    "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
                        .into(),
                ),
                (
                    "title".into(),
                    "input format must be 'xxxxxxxx-xxxx-Mxxx-Nxxx-xxxxxxxxxxxx'".into(),
                ),
            ]);
            input
        }
        // sea_orm::ColumnType::Custom(DynIden),
        // sea_orm::ColumnType::Enum {
        //     name: DynIden,
        //     variants: Vec<DynIden>,
        // },
        // sea_orm::ColumnType::Array(o) => {}
        // sea_orm::ColumnType::Cidr => {
        //     input.attributes = HashMap::from_iter([
        //         // TODO: ("pattern".into(), "[0-9a-fA-F]*".into()),
        //         ("title".into(), "input format must be cidr".into()),
        //     ]);
        //     input
        // }
        // sea_orm::ColumnType::Inet => {
        //     input.attributes = HashMap::from_iter([
        //         // TODO: ("pattern".into(), "[0-9a-fA-F]*".into()),
        //         ("title".into(), "input format must be inet".into()),
        //     ]);
        //     input
        // }
        // sea_orm::ColumnType::MacAddr => {
        //     input.attributes = HashMap::from_iter([
        //         // TODO: ("pattern".into(), "[0-9a-fA-F]*".into()),
        //         ("title".into(), "input format must be macaddr".into()),
        //     ]);
        //     input
        // }
        _ => {
            warn!("Unsuported column type: {:?}", field.column_type);
            return Err(Box::new(CustomError::new("Unsupported column type")));
        }
    })
}
