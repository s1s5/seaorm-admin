use std::collections::HashMap;

pub use async_trait::async_trait;
pub use sea_orm;
pub use sea_orm::Iden;
use sea_orm::{ColumnDef, Condition, DatabaseConnection, DatabaseTransaction};
mod admin;
#[cfg(feature = "with-axum")]
pub mod axum_admin;
mod error;
mod field;
mod filter;
mod json;
mod key;
mod parse;
#[cfg(feature = "with-rocket")]
pub mod rocket_admin;
pub mod templates;
#[cfg(test)]
mod tests;

pub use admin::*;
pub use admin_macro::ModelAdmin;
pub use error::*;
pub use field::*;
pub use filter::*;
pub use json::*;
pub use key::*;
pub use parse::*;

pub type Result<T> = std::result::Result<T, anyhow::Error>;
pub type Json = serde_json::Value;

#[derive(Debug, Clone)]
pub struct ListQuery {
    pub filter: HashMap<String, Vec<String>>,
    pub queries: Vec<String>,
    pub ordering: Vec<(String, sea_orm::Order)>,
    pub offset: u64,
    pub limit: u64,
}

#[derive(Debug, Clone)]
pub struct ListParam {
    pub cond: Condition,
    pub ordering: Vec<(String, sea_orm::Order)>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[async_trait]
pub trait ModelAdminTrait {
    fn get_table_name(&self) -> &str;
    fn get_list_per_page(&self) -> u64;
    fn get_columns(&self) -> Vec<(String, ColumnDef)>;
    fn get_primary_keys(&self) -> Vec<String>;

    fn to_str(&self, value: &Json) -> Result<String>;

    fn json_to_key(&self, value: &Json) -> Result<String>;

    fn key_to_json(&self, key: &str) -> Result<Json>;

    fn list_display(&self) -> Vec<String>;

    fn get_form_fields(&self) -> Vec<AdminField>;

    async fn list(&self, conn: &DatabaseConnection, param: &ListParam) -> Result<(u64, Vec<Json>)>;
    async fn get(&self, conn: &DatabaseConnection, cond: &Condition) -> Result<Option<Json>>;
    async fn insert(&self, conn: &DatabaseTransaction, value: &Json) -> Result<Json>;
    async fn update(&self, conn: &DatabaseTransaction, value: &Json) -> Result<Json>;
    async fn delete(&self, conn: &DatabaseTransaction, cond: &Condition) -> Result<u64>;
}
