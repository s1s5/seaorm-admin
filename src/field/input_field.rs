use crate::{field::FieldTrait, json_force_str, templates::AdminFormInput, Admin, Json, Result};
use askama::DynTemplate;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct InputField(AdminFormInput);

impl InputField {
    pub fn new_with_type(name: &str, r#type: &str) -> Self {
        InputField(AdminFormInput {
            name: name.into(),
            label: name.into(),
            r#type: r#type.into(),
            value: None,
            help_text: None,
            disabled: false,
            attributes: HashMap::new(),
        })
    }

    pub fn new_for_char(name: &str, max_length: u32) -> Self {
        let mut input = Self::new_with_type(name, "text");
        input.0.attributes =
            HashMap::from_iter([("maxlength".to_string(), max_length.to_string())]);
        input
    }

    pub fn new_for_int(name: &str) -> Self {
        let mut input = Self::new_with_type(name, "number");
        input.0.attributes = HashMap::from_iter([("step".into(), "1".into())]);
        input
    }

    pub fn new_for_float(name: &str) -> Self {
        let mut input = Self::new_with_type(name, "number");
        input.0.attributes = HashMap::from_iter([("step".into(), "auto".into())]);
        input
    }

    pub fn new_for_uuid(name: &str) -> Self {
        let mut input = Self::new_with_type(name, "text");
        input.0.attributes = HashMap::from_iter([
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
}

#[async_trait]
impl FieldTrait for InputField {
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
        template.value = value.map(|x| json_force_str(&x));
        template.disabled = disabled;
        Ok(Box::new(template))
    }
}
