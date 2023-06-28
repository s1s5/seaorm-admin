use super::AdminField;
use super::{
    foreign_key_field::{extract_table_name, identity_to_vec_string},
    RelationTrait,
};
use crate::templates::AdminFormAutoComplete;
use crate::{extract_cols_from_relation_def, json_extract_prefixed};
use crate::{
    templates::{RelationForm, RelationFormRow, RelationFormRowField},
    Admin, Json, Result,
};
use askama::DynTemplate;
use async_trait::async_trait;
use sea_orm::RelationDef;
use std::collections::HashSet;

pub struct ManyToMany {
    name: String,
    from_def: RelationDef,
    to_def: RelationDef,
}

impl ManyToMany {
    pub fn new(name: &str, from_def: RelationDef, to_def: RelationDef) -> Self {
        ManyToMany {
            name: name.to_string(),
            from_def,
            to_def,
        }
    }
}

#[async_trait]
impl RelationTrait for ManyToMany {
    async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        prefix: &str,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        println!("fr:{:?} to:{:?}", self.from_def, self.to_def);

        Ok(Box::new(AdminFormAutoComplete {
            name: self.name.clone(),
            label: self.name.clone(),
            choices: vec![],
            help_text: None,
            disabled: disabled,
            to_table: extract_table_name(&self.to_def.to_tbl)?,
            cols: extract_cols_from_relation_def(&self.to_def)?,
            nullable: true,
            multiple: true,
        }))
    }

    async fn commit(&self, admin: &Admin, parent_value: &Json) -> Result<Json> {
        Err(anyhow::anyhow!("todo"))
    }
}
