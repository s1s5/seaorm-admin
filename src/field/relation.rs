use super::AdminField;
use super::{
    foreign_key_field::{extract_table_name, identity_to_vec_string},
    RelationTrait,
};
use crate::{
    templates::{RelationForm, RelationFormRow, RelationFormRowField},
    Admin, Json, Result,
};
use askama::DynTemplate;
use async_trait::async_trait;
use sea_orm::RelationDef;
use std::collections::HashSet;

pub struct Relation {
    name: String,
    def: RelationDef,
}

impl Relation {
    pub fn new(name: &str, def: RelationDef) -> Self {
        Relation {
            name: name.into(),
            def,
        }
    }
}

#[async_trait]
impl RelationTrait for Relation {
    async fn get_template(
        &self,
        admin: &Admin,
        parent_value: Option<&Json>,
        prefix: &str,
        disabled: bool,
    ) -> Result<Box<dyn DynTemplate + Send>> {
        let model = admin
            .get_model(&extract_table_name(&self.def.from_tbl)?)
            .ok_or(anyhow::anyhow!("no related table found"))?;
        let fields = model.get_form_fields();
        let cols: HashSet<String> = identity_to_vec_string(&self.def.from_col)
            .into_iter()
            .collect();

        let mut rows = vec![];
        if let Some(parent_value) = parent_value {
            let m: serde_json::Map<String, Json> = identity_to_vec_string(&self.def.from_col)
                .into_iter()
                .zip(identity_to_vec_string(&self.def.to_col).into_iter())
                .map(|(fr, to)| (fr.clone(), parent_value.get(&to)))
                .filter(|x| x.1.filter(|x| !x.is_null()).is_some())
                .map(|x| (x.0, x.1.unwrap().clone()))
                .collect();

            let jv_list = model
                .list_by_key(&admin.get_connection(), &Json::Object(m))
                .await?;

            let pkeys: HashSet<String> = model.get_primary_keys().into_iter().collect();

            for (i, jv) in jv_list.iter().enumerate() {
                let row_prefix = format!("{}{}.{}.", prefix, self.name, i,);
                let mut fv = vec![];
                for f in fields.iter() {
                    let is_pkey = match f {
                        AdminField::Field(x) => {
                            if x.fields().into_iter().any(|i| cols.contains(&i)) {
                                continue;
                            }
                            if x.fields().into_iter().any(|i| pkeys.contains(&i)) {
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };
                    // println!("name={}", f.name());
                    // // TOOD: auto_completeが弾けてない
                    // if cols.contains(f.name()) {
                    //     continue;
                    // }
                    fv.push(RelationFormRowField {
                        is_pkey,
                        field: f
                            .get_template(admin, Some(jv), &row_prefix, disabled || is_pkey)
                            .await?,
                    });
                }
                rows.push(RelationFormRow { fields: fv });
            }
        }

        Ok(Box::new(RelationForm {
            name: format!("{}{}", prefix, self.name),
            rows: rows,
        }))
    }
}
