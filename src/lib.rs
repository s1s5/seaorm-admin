use std::collections::HashMap;

pub use async_trait::async_trait;
use sea_orm::RelationDef;
pub use sea_orm::{DatabaseConnection, EntityName, Iden, ModelTrait, PrimaryKeyToColumn};

mod admin;
mod error;
mod field;
mod filter;
mod json;
mod key;
mod parse;
pub mod rocket_admin;
pub mod templates;

pub use admin::*;
pub use admin_macro::ModelAdmin;
pub use error::*;
pub use field::*;
pub use filter::*;
pub use json::*;
pub use key::*;
pub use parse::*;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;
pub type Json = serde_json::Value;

#[derive(Debug, Clone)]
pub struct ListQuery {
    pub filter: HashMap<String, Vec<String>>,
    pub queries: Vec<String>,
    pub offset: u64,
    pub limit: u64,
}

#[async_trait]
pub trait ModelAdminTrait {
    fn get_table_name(&self) -> &str;
    fn get_list_per_page(&self) -> u64;

    fn to_str(&self, value: &Json) -> Result<String>;

    fn json_to_key(&self, value: &Json) -> Result<String>;

    fn key_to_json(&self, key: &str) -> Result<Json>;

    fn list_display(&self) -> Vec<String>;

    fn get_auto_complete(&self) -> Vec<RelationDef>;

    fn get_create_form_fields(&self) -> Vec<AdminField>;

    fn get_update_form_fields(&self) -> Vec<AdminField>;

    async fn list(&self, conn: &DatabaseConnection, query: &ListQuery) -> Result<(u64, Vec<Json>)>;

    async fn get(&self, conn: &DatabaseConnection, key: Json) -> Result<Option<Json>>;

    async fn insert(&self, conn: &DatabaseConnection, value: Json) -> Result<Json>;

    async fn update(&self, conn: &DatabaseConnection, value: Json) -> Result<Json>;

    async fn delete(&self, conn: &DatabaseConnection, value: Json) -> Result<u64>;
}