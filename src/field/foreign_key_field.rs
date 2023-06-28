use std::collections::HashMap;

use super::FieldTrait;
use crate::{
    create_cond_from_json, json_force_str,
    templates::{self, AdminFormAutoComplete},
    Admin, CustomError, Json, Result,
};
use askama::DynTemplate;
use async_trait::async_trait;
use itertools::Itertools;
use sea_orm::{sea_query::IdenList, ColumnTrait, RelationDef};

pub struct ForeignKeyField(AdminFormAutoComplete);

pub fn identity_to_vec_string(ident: &sea_orm::Identity) -> Vec<String> {
    match ident {
        sea_orm::Identity::Unary(i0) => {
            vec![i0.to_string()]
        }
        sea_orm::Identity::Binary(i0, i1) => {
            vec![i0.to_string(), i1.to_string()]
        }
        sea_orm::Identity::Ternary(i0, i1, i2) => {
            vec![i0.to_string(), i1.to_string(), i2.to_string()]
        }
    }
}

pub fn extract_table_name(ident: &sea_orm::sea_query::TableRef) -> Result<String> {
    match ident {
        sea_orm::sea_query::TableRef::Table(t) => Ok(t.to_string()),
        _ => Err(anyhow::anyhow!("Unsupported Type")),
    }
}

fn relation_def_to_form_name(def: &sea_orm::RelationDef) -> Result<String> {
    let fr = identity_to_vec_string(&def.from_col);
    let to = identity_to_vec_string(&def.to_col);
    Ok(fr
        .iter()
        .zip(to.iter())
        .map(|(a, b)| format!("{}_{}", a, b))
        .join("-"))
}

fn relation_def_to_form_label(def: &sea_orm::RelationDef) -> Result<String> {
    let fr_table_name = extract_table_name(&def.from_tbl)?;
    let to_table_name = extract_table_name(&def.to_tbl)?;
    let fr = identity_to_vec_string(&def.from_col);
    let to = identity_to_vec_string(&def.to_col);
    Ok(fr
        .iter()
        .zip(to.iter())
        .map(|(a, b)| format!("{}.{} => {}.{}", fr_table_name, a, to_table_name, b))
        .join("; "))
}

pub fn extract_cols_from_relation_def(
    def: &sea_orm::RelationDef,
) -> Result<Vec<templates::AdminFormAutoCompleteCol>> {
    let fr = identity_to_vec_string(&def.from_col);
    let to = identity_to_vec_string(&def.to_col);
    Ok(fr
        .into_iter()
        .zip(to.into_iter())
        .map(|(f, t)| templates::AdminFormAutoCompleteCol {
            value: vec![],
            from_col: f,
            to_col: t,
        })
        .collect())
}

pub fn relation_def_is_nullable<T>(def: &sea_orm::RelationDef, columns: &Vec<T>) -> bool
where
    T: ColumnTrait,
{
    let m: HashMap<String, bool> = columns
        .iter()
        .map(|x| (x.to_string(), x.def().is_null()))
        .collect();
    def.from_col
        .clone()
        .into_iter()
        .any(|x| m.get(&x.to_string()).map(|x| x.clone()).unwrap_or(false))
}

impl ForeignKeyField {
    pub fn new(rel_def: &RelationDef, nullable: bool) -> Result<Self> {
        Ok(ForeignKeyField(AdminFormAutoComplete {
            name: relation_def_to_form_name(rel_def)?,
            label: relation_def_to_form_label(rel_def)?,
            choices: vec![],
            help_text: None,
            disabled: false,
            to_table: extract_table_name(&rel_def.to_tbl)?,
            cols: extract_cols_from_relation_def(rel_def)?,
            nullable,
            multiple: false,
        }))
    }
}

#[async_trait]
impl FieldTrait for ForeignKeyField {
    fn fields(&self) -> Vec<String> {
        self.0.cols.iter().map(|x| x.from_col.clone()).collect()
    }

    async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        prefix: &str,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let mut template = self.0.clone();
        if let Some(parent_value) = parent_value {
            let tm = admin
                .get_model(&self.0.to_table)
                .ok_or(CustomError::new("no table found"))?;
            let m: serde_json::Map<String, Json> = self
                .0
                .cols
                .iter()
                .map(|k| (k.to_col.clone(), parent_value.get(&k.from_col)))
                .filter(|x| x.1.filter(|x| !x.is_null()).is_some())
                .map(|x| (x.0, x.1.unwrap().clone()))
                .collect();
            let cond = create_cond_from_json(
                &tm.get_columns().iter().map(|x| x.0.clone()).collect(),
                &Json::Object(m),
                false,
            )?;
            let tr = tm.get(&admin.get_connection(), &cond).await.unwrap_or(None);
            template.cols = template
                .cols
                .iter()
                .map(|x| templates::AdminFormAutoCompleteCol {
                    value: vec![super::tool::get_value(Some(parent_value), &x.from_col)
                        .map(|x| json_force_str(x))
                        .unwrap_or("".to_string())],
                    from_col: x.from_col.clone(),
                    to_col: x.to_col.clone(),
                })
                .collect();

            if let Some(tr) = tr {
                template.choices = vec![templates::AdminFormAutoCompleteChoice {
                    label: tm.to_str(&tr)?,
                    value: tm.json_to_key(&tr)?,
                    json_str: serde_json::to_string(&tr)?,
                }];
            }
        };
        template.name = format!("{}{}", prefix, template.name);
        template.disabled = disabled;
        Ok(Box::new(template))
    }
}
