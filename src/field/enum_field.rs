use super::{AdminField, FieldTrait};
use crate::{json_force_str, templates::AdminFormSelect, Admin, Json, Result};
use askama::DynTemplate;
use async_trait::async_trait;
use sea_orm::ColumnTrait;
use std::collections::HashMap;
use sea_orm::ActiveEnum;
pub struct EnumField(AdminFormSelect);

pub fn enum_field<C, T>(col: C, it: T) -> AdminField
where
    C: ColumnTrait,
    T: Iterator,
    <T as Iterator>::Item: std::fmt::Debug,
    <T as Iterator>::Item: sea_orm::ActiveEnum,
    <<T as Iterator>::Item as ActiveEnum>::Value: std::fmt::Display,
{
    AdminField::Field(Box::new(EnumField::from_enum(col, it)))
}

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
        <T as Iterator>::Item: std::fmt::Debug,
        <T as Iterator>::Item: sea_orm::ActiveEnum,
        <<T as Iterator>::Item as ActiveEnum>::Value: std::fmt::Display,
    {
        Self::new(
            &col.to_string(),
            it.map(|x| {
                (
                    format!("{}", x.to_value()),
                    format!("{:?}", x),
                )
            })
            .collect(),
        )
    }
}

#[async_trait]
impl FieldTrait for EnumField {
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
        let mut template = self.0.clone();
        template.name = format!("{}{}", prefix, template.name);
        template.value = value.map(|x| json_force_str(&x)).unwrap_or("".to_string());
        template.disabled = disabled;
        Ok(Box::new(template))
    }
}
