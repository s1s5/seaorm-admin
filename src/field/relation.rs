use super::RelationTrait;
use crate::{templates::RelationForm, Admin, Json, Result};
use askama::DynTemplate;
use async_trait::async_trait;

pub struct Relation {
    name: String,
}

impl Relation {
    pub fn new(name: &str) -> Self {
        Relation { name: name.into() }
    }
}

#[async_trait]
impl RelationTrait for Relation {
    fn name(&self) -> &str {
        &self.name
    }

    async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        Ok(Box::new(RelationForm {}))
    }
}
