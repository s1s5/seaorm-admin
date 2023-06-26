use super::FieldTrait;
use crate::{json_force_str, templates::AdminFormSelect, Admin, Json, Result};
use askama::DynTemplate;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct EnumField(AdminFormSelect);

impl EnumField {
    pub fn new(name: &str, choices: Vec<(String, String)>) -> Self {
        EnumField(AdminFormSelect {
            name: name.into(),
            label: name.into(),
            help_text: None,
            value: "".into(),
            disabled: false,
            attributes: HashMap::new(),
            choices: choices,
        })
    }

    pub fn from_enum<T>(name: &str, it: T) -> Self
    where
        T: Iterator,
        <T as Iterator>::Item: std::fmt::Debug + std::fmt::Display,
    {
        Self::new(
            name,
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

#[async_trait]
impl FieldTrait for EnumField {
    fn name(&self) -> &str {
        &self.0.name
    }

    async fn get_template(
        &self,
        _admin: &Admin,
        parent_value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let value = super::tool::get_value(parent_value, &self.0.name);
        let mut template = self.0.clone();
        template.value = value.map(|x| json_force_str(&x)).unwrap_or("".to_string());
        template.disabled = disabled;
        Ok(Box::new(template))
    }
}
