mod checkbox_field;
mod date_time_field;
mod default_field;
mod enum_field;
mod foreign_key_field;
mod input_field;
mod relation;
mod textarea_field;
mod timestamp_field;
mod tool;
use super::{Json, Result};
use crate::Admin;
use askama::DynTemplate;
use async_trait::async_trait;

pub use checkbox_field::CheckboxField;
pub use date_time_field::DateTimeField;
pub use default_field::get_default_field;
pub use enum_field::{enum_field, EnumField};
pub use foreign_key_field::{
    extract_cols_from_relation_def, relation_def_is_nullable, ForeignKeyField,
};
pub use input_field::InputField;
pub use textarea_field::TextareaField;
pub use timestamp_field::TimestampField;

pub enum AdminField {
    Field(Box<dyn FieldTrait + Send + Sync>),
    Relation(Box<dyn RelationTrait + Send + Sync>),
}

impl AdminField {
    pub fn name(&self) -> &str {
        match &self {
            AdminField::Field(f) => f.name(),
            AdminField::Relation(f) => f.name(),
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
            AdminField::Relation(f) => f.get_template(admin, parent_value, disabled).await,
        }
    }
}

#[async_trait]
pub trait FieldTrait {
    fn name(&self) -> &str;
    async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>>;
}

#[async_trait]
pub trait RelationTrait {
    fn name(&self) -> &str;
    async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>>;
}
