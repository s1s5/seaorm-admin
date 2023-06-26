use super::FieldTrait;
use crate::{json_force_str, templates::AdminFormSelect, Admin, Json, Result};
use askama::DynTemplate;
use async_trait::async_trait;
use sea_orm::ColumnTrait;
use std::collections::HashMap;

pub struct EnumField(AdminFormSelect);

#[macro_export]
macro_rules! enum_field {
    ($col:path, $e:expr) => {
        seaorm_admin::AdminField::Field(Box::new(seaorm_admin::EnumField::from_enum($col, $e)))
    };
}

pub use enum_field;

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

    pub fn from_enum<C, T>(col: C, it: T) -> Self
    where
        C: ColumnTrait,
        T: Iterator,
        <T as Iterator>::Item: std::fmt::Debug + std::fmt::Display,
    {
        Self::new(
            &col.to_string(),
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
