use crate::{field::FieldTrait, json_force_str, templates::AdminFormTextarea, Admin, Json, Result};
use askama::DynTemplate;
use async_trait::async_trait;

pub struct TextareaField(AdminFormTextarea);

impl TextareaField {
    pub fn new(name: &str) -> Self {
        TextareaField(AdminFormTextarea {
            name: name.into(),
            label: name.into(),
            value: None,
            help_text: None,
            disabled: false,
        })
    }
}

#[async_trait]
impl FieldTrait for TextareaField {
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
        template.value = value.map(|x| json_force_str(&x));
        template.disabled = disabled;
        Ok(Box::new(template))
    }
}
