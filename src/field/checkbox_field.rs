use crate::{
    field::FieldTrait,
    templates::{AdminFormCheckbox, AdminFormInput},
    Admin, Json, Result,
};
use askama::DynTemplate;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct CheckboxField(AdminFormInput);
impl CheckboxField {
    pub fn new(name: &str) -> Self {
        CheckboxField(AdminFormInput {
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
impl FieldTrait for CheckboxField {
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
        Ok(Box::new(AdminFormCheckbox {
            name: format!("{}{}", prefix, self.0.name.clone()),
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
