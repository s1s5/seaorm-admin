use super::FieldTrait;
use crate::{
    json_force_str,
    templates::{
        self, AdminFormAutoComplete, AdminFormCheckbox, AdminFormDatetimeInput,
        AdminFormDatetimeInputValue, AdminFormInput, AdminFormSelect, AdminFormTextarea,
    },
    Admin, CustomError, Json, Result,
};
use askama::DynTemplate;
use async_trait::async_trait;
use chrono::Timelike;
use itertools::Itertools;
use log::warn;
use sea_orm::{ColumnDef, ColumnTrait, RelationDef};
use std::collections::HashMap;

pub struct DefaultField {
    name: String,
    widget: Box<dyn WidgetTrait + Send + Sync>,
}

impl DefaultField {
    pub fn create_from<T>(column: &T) -> Result<Self>
    where
        T: ColumnTrait,
    {
        let name = column.to_string();
        Ok(DefaultField {
            widget: get_default_widget(&name, column.def())?,
            name: name,
        })
    }
}

#[async_trait]
impl FieldTrait for DefaultField {
    fn name(&self) -> &str {
        &self.name
    }

    async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let value = parent_value
            .map(|x| x.as_object())
            .unwrap_or(None)
            .map(|y| y.get(&self.name))
            .unwrap_or(None);
        self.widget.get_template(value, disabled)
    }
}

// impl DefaultField {
//     pub fn create_from<T>(column: &T, editable: bool) -> Self
//     where
//         T: ColumnTrait,
//     {
//         AdminField {
//             column_type: column.def().get_column_type().clone(),
//             name: column.to_string(),
//             editable: editable,
//             hidden: false,
//             required: false,
//             help_text: None,
//             nullable: column.def().is_null(),
//         }
//     }
// }

fn get_admin_form_input(name: &str) -> AdminFormInput {
    AdminFormInput {
        name: name.into(),
        label: name.into(),
        r#type: "text".into(),
        value: None,
        help_text: None,
        disabled: false,
        attributes: HashMap::new(),
    }
}

fn get_admin_form_textarea(name: &str) -> AdminFormTextarea {
    AdminFormTextarea {
        name: name.into(),
        label: name.into(),
        value: None,
        help_text: None,
        disabled: false,
    }
}

fn get_default_widget(name: &str, def: ColumnDef) -> Result<Box<dyn WidgetTrait + Send + Sync>> {
    Ok(match &def.get_column_type() {
        sea_orm::ColumnType::Char(o) | sea_orm::ColumnType::String(o) => {
            if let Some(max_length) = o {
                InputWidget::new_for_char(get_admin_form_input(name), *max_length)
            } else {
                Box::new(get_admin_form_textarea(name))
            }
        }
        sea_orm::ColumnType::Text | sea_orm::ColumnType::Json | sea_orm::ColumnType::JsonBinary => {
            Box::new(get_admin_form_textarea(name))
        }
        sea_orm::ColumnType::TinyInteger
        | sea_orm::ColumnType::SmallInteger
        | sea_orm::ColumnType::Integer
        | sea_orm::ColumnType::BigInteger
        | sea_orm::ColumnType::TinyUnsigned
        | sea_orm::ColumnType::SmallUnsigned
        | sea_orm::ColumnType::Unsigned
        | sea_orm::ColumnType::BigUnsigned => InputWidget::new_for_int(get_admin_form_input(name)),
        sea_orm::ColumnType::Float
        | sea_orm::ColumnType::Double
        | sea_orm::ColumnType::Decimal(_)
        | sea_orm::ColumnType::Money(_) => InputWidget::new_for_float(get_admin_form_input(name)),
        sea_orm::ColumnType::DateTime => Box::new(DateTimeWidget(get_admin_form_input(name))),
        sea_orm::ColumnType::TimestampWithTimeZone => {
            Box::new(TimestampWidget(get_admin_form_input(name)))
        }

        sea_orm::ColumnType::Time => InputWidget::new_for_type(get_admin_form_input(name), "time"),
        sea_orm::ColumnType::Date => InputWidget::new_for_type(get_admin_form_input(name), "date"),
        sea_orm::ColumnType::Year(_o) => {
            InputWidget::new_for_type(get_admin_form_input(name), "number")
        }
        sea_orm::ColumnType::Binary(_) | sea_orm::ColumnType::VarBinary(_) => {
            // pattern="^[A-Za-z0-9+/]{4}*[A-Za-z0-9+/]{4}([A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$"
            Box::new(get_admin_form_textarea(name))
        }
        sea_orm::ColumnType::Boolean => Box::new(CheckboxWidget(get_admin_form_input(name))),
        sea_orm::ColumnType::Uuid => InputWidget::new_for_uuid(get_admin_form_input(name)),
        _ => {
            warn!("Unsuported column type: {:?}", def.get_column_type());
            return Err(anyhow::anyhow!("Unsupported column type"));
        }
    })
}

trait WidgetTrait {
    fn get_template(
        &self,
        value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>>;
}

struct InputWidget(AdminFormInput);
impl InputWidget {
    fn new_for_char(
        mut input: AdminFormInput,
        max_length: u32,
    ) -> Box<dyn WidgetTrait + Send + Sync> {
        input.r#type = "text".into();
        input.attributes = HashMap::from_iter([("maxlength".to_string(), max_length.to_string())]);
        Box::new(InputWidget(input))
    }

    fn new_for_int(mut input: AdminFormInput) -> Box<dyn WidgetTrait + Send + Sync> {
        input.r#type = "number".into();
        input.attributes = HashMap::from_iter([("step".into(), "1".into())]);
        Box::new(InputWidget(input))
    }

    fn new_for_float(mut input: AdminFormInput) -> Box<dyn WidgetTrait + Send + Sync> {
        input.r#type = "number".into();
        input.attributes = HashMap::from_iter([("step".into(), "auto".into())]);
        Box::new(InputWidget(input))
    }
    fn new_for_uuid(mut input: AdminFormInput) -> Box<dyn WidgetTrait + Send + Sync> {
        input.r#type = "text".into();
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
        Box::new(InputWidget(input))
    }
    fn new_for_type(mut input: AdminFormInput, r#type: &str) -> Box<dyn WidgetTrait + Send + Sync> {
        input.r#type = r#type.into();
        Box::new(InputWidget(input))
    }
}

impl WidgetTrait for InputWidget {
    fn get_template(
        &self,
        value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let mut template = self.0.clone();
        template.value = value.map(|x| json_force_str(&x));
        template.disabled = disabled;
        Ok(Box::new(template))
    }
}

impl WidgetTrait for AdminFormTextarea {
    fn get_template(
        &self,
        value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let mut template = self.clone();
        template.value = value.map(|x| json_force_str(&x));
        template.disabled = disabled;
        Ok(Box::new(template))
    }
}

struct DateTimeWidget(AdminFormInput);

impl WidgetTrait for DateTimeWidget {
    fn get_template(
        &self,
        value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let mut template = AdminFormDatetimeInput {
            name: self.0.name.clone(),
            label: self.0.label.clone(),
            value: None,
            with_timezone: false,
            help_text: self.0.help_text.clone(),
            disabled: disabled,
        };
        let value = value.map(|x| json_force_str(&x));
        let value = if let Some(value) = &value {
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
            None
        };
        template.value = value;
        Ok(Box::new(template))
    }
}

struct TimestampWidget(AdminFormInput);
impl WidgetTrait for TimestampWidget {
    fn get_template(
        &self,
        value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let mut template = AdminFormDatetimeInput {
            name: self.0.name.clone(),
            label: self.0.label.clone(),
            value: None,
            with_timezone: true,
            help_text: self.0.help_text.clone(),
            disabled,
        };
        let value = value.map(|x| json_force_str(&x));
        let value = if let Some(value) = &value {
            let v: Option<chrono::NaiveDateTime> =
                serde_json::from_value(Json::String(value.clone())).ok();
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
        } else {
            None
        };
        template.value = value;
        Ok(Box::new(template))
    }
}

struct CheckboxWidget(AdminFormInput);
impl WidgetTrait for CheckboxWidget {
    fn get_template(
        &self,
        value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        Ok(Box::new(AdminFormCheckbox {
            name: self.0.name.clone(),
            label: self.0.label.clone(),
            checked: value
                .unwrap_or(&Json::Bool(false))
                .as_bool()
                .unwrap_or(false),
            help_text: None,
            disabled: disabled,
        }))
    }
}

pub struct EnumWidgetFactory {
    choices: Vec<(String, String)>,
}

pub struct EnumWidget(AdminFormSelect);

impl EnumWidgetFactory {
    pub fn new(choices: Vec<(String, String)>) -> Self {
        EnumWidgetFactory { choices: choices }
    }

    pub fn from_enum<T>(it: T) -> Self
    where
        T: Iterator,
        <T as Iterator>::Item: std::fmt::Debug + std::fmt::Display,
    {
        Self::new(
            it.map(|x| {
                (
                    x.to_string().trim_matches('\'').to_string(),
                    format!("{:?}", x),
                )
            })
            .collect(),
        )
    }
}

impl WidgetTrait for EnumWidget {
    fn get_template(
        &self,
        value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let mut template = self.0.clone();
        template.value = value.map(|x| json_force_str(&x)).unwrap_or("".to_string());
        template.disabled = disabled;
        Ok(Box::new(template))
    }
}
