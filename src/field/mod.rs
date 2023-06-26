pub mod default_field;
pub mod foreign_key_field;

use super::{Json, Result};
use crate::Admin;
use askama::DynTemplate;
use async_trait::async_trait;

pub enum AdminField {
    Field(Box<dyn FieldTrait + Send + Sync>),
    OneToOne(Box<dyn RelationTrait + Send + Sync>),
    OneToMany(Box<dyn RelationTrait + Send + Sync>),
    ManyToMany(Box<dyn RelationTrait + Send + Sync>),
}

impl AdminField {
    pub fn name(&self) -> &str {
        match &self {
            AdminField::Field(f) => f.name(),
            AdminField::OneToOne(f) => f.name(),
            AdminField::OneToMany(f) => f.name(),
            AdminField::ManyToMany(f) => f.name(),
        }
    }

    pub async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        match &self {
            AdminField::Field(f) => f.get_template(admin, parent_value, disabled).await,
            AdminField::OneToOne(f) => f.get_template(admin, parent_value, disabled).await,
            AdminField::OneToMany(f) => f.get_template(admin, parent_value, disabled).await,
            AdminField::ManyToMany(f) => f.get_template(admin, parent_value, disabled).await,
        }
    }
}

#[async_trait]
trait FieldTrait {
    fn name(&self) -> &str;
    async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>>;
}

#[async_trait]
trait RelationTrait {
    fn name(&self) -> &str;
    async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>>;
}
