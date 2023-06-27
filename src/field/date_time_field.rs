use crate::{
    field::FieldTrait,
    json_force_str,
    templates::{AdminFormDatetimeInput, AdminFormDatetimeInputValue, AdminFormInput},
    Admin, Json, Result,
};
use askama::DynTemplate;
use async_trait::async_trait;
use chrono::Timelike;
use std::collections::HashMap;

pub struct DateTimeField(AdminFormInput);

impl DateTimeField {
    pub fn new(name: &str) -> Self {
        DateTimeField(AdminFormInput {
            name: name.into(),
            label: name.into(),
            r#type: "text".into(),
            value: None,
            help_text: None,
            disabled: false,
            attributes: HashMap::new(),
        })
    }
}

#[async_trait]
impl FieldTrait for DateTimeField {
    fn fields(&self) -> Vec<String> {
        vec![self.0.name.clone()]
    }

    async fn get_template(
        &self,
        _admin: &Admin,
        parent_value: Option<&Json>,
        prefix: &str,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let value = super::tool::get_value(parent_value, &self.0.name);
        let mut template = AdminFormDatetimeInput {
            name: format!("{}{}", prefix, self.0.name.clone()),
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
