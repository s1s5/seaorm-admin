use super::AdminField;
use super::{
    foreign_key_field::{extract_table_name, identity_to_vec_string},
    RelationTrait,
};
use crate::{create_cond_from_json, json_extract_prefixed};
use crate::{
    templates::{RelationForm, RelationFormRow, RelationFormRowField},
    Admin, Json, Result,
};
use askama::DynTemplate;
use async_trait::async_trait;
use sea_orm::{DatabaseTransaction, RelationDef};
use std::collections::HashSet;

pub struct Relation {
    name: String,
    def: RelationDef,
    multiple: bool,
}

impl Relation {
    pub fn new(name: &str, def: RelationDef, multiple: bool) -> Self {
        Relation {
            name: name.into(),
            def,
            multiple,
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

        let pkeys: HashSet<String> = model.get_primary_keys().into_iter().collect();
        let mut rows = vec![];
        if let Some(parent_value) = parent_value {
            let m: serde_json::Map<String, Json> = identity_to_vec_string(&self.def.from_col)
                .into_iter()
                .zip(identity_to_vec_string(&self.def.to_col).into_iter())
                .map(|(fr, to)| (fr.clone(), parent_value.get(&to)))
                .filter(|x| x.1.filter(|x| !x.is_null()).is_some())
                .map(|x| (x.0, x.1.unwrap().clone()))
                .collect();

            let cond = create_cond_from_json(
                &model.get_columns().iter().map(|x| x.0.clone()).collect(),
                &Json::Object(m),
                false,
            )?;

            let (_, jv_list) = model
                .list(
                    &admin.get_connection(),
                    &crate::ListParam {
                        cond: cond,
                        ordering: vec![],
                        offset: None,
                        limit: None,
                    },
                )
                .await?;

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
                rows.push(RelationFormRow {
                    is_update: true,
                    fields: fv,
                });
            }
        }
        if (!self.multiple) && rows.len() == 0 {
            let row_prefix = format!("{}{}.{}.", prefix, self.name, 0,);
            let mut fv = vec![];
            for f in fields.iter() {
                match f {
                    AdminField::Field(x) => {
                        if x.fields().into_iter().any(|i| cols.contains(&i)) {
                            continue;
                        }
                        if x.fields().into_iter().any(|i| pkeys.contains(&i)) {
                            continue;
                        }
                    }
                    _ => {}
                };

                fv.push(RelationFormRowField {
                    is_pkey: false,
                    field: f.get_template(admin, None, &row_prefix, disabled).await?,
                });
            }
            rows.push(RelationFormRow {
                is_update: false,
                fields: fv,
            })
        }

        let mut template_fields = vec![];
        for f in fields.iter() {
            let is_pkey = match f {
                AdminField::Field(x) => {
                    if x.fields().into_iter().any(|i| cols.contains(&i)) {
                        continue;
                    }
                    if x.fields().into_iter().any(|i| pkeys.contains(&i)) {
                        continue;
                    } else {
                        false
                    }
                }
                _ => false,
            };
            let row_prefix = format!("{}{}.{}.", prefix, self.name, "${index}",);
            template_fields.push(
                f.get_template(admin, None, &row_prefix, disabled || is_pkey)
                    .await?,
            );
        }

        Ok(Box::new(RelationForm {
            name: format!("{}{}", prefix, self.name),
            template_fields,
            rows,
            multiple: self.multiple,
        }))
    }

    async fn commit(
        &self,
        admin: &Admin,
        parent_value: &Json,
        txn: &DatabaseTransaction,
    ) -> Result<Json> {
        let parent_object = parent_value
            .as_object()
            .ok_or(anyhow::anyhow!("invalid json"))?;
        let state = parent_object
            .get(&format!("{}.state", self.name))
            .ok_or(anyhow::anyhow!("state not found"))?
            .as_str()
            .ok_or(anyhow::anyhow!("state must be string"))?;

        let model = admin
            .get_model(&extract_table_name(&self.def.from_tbl)?)
            .ok_or(anyhow::anyhow!("table not found"))?;

        for (i, op) in state.split(",").filter(|x| (*x).len() > 0).enumerate() {
            let mut data = json_extract_prefixed(parent_value, &format!("{}.{}.", self.name, i))?;
            match &op {
                &"C" => {
                    let data_object = data.as_object_mut().unwrap();
                    identity_to_vec_string(&self.def.from_col)
                        .into_iter()
                        .zip(identity_to_vec_string(&self.def.to_col).into_iter())
                        .for_each(|(fr, to)| {
                            if let Some(v) = parent_object.get(&to) {
                                data_object.insert(fr, v.clone());
                            }
                        });
                    // println!("create {:?}", data);
                    admin.create(model, &data, Some(txn)).await?;
                }
                &"U" => {
                    //println!("update {:?}", data);
                    admin.update(model, &data, Some(txn)).await?;
                }
                &"D" => {
                    // println!("delete {:?}", data);
                    admin.delete(model, &data, Some(txn)).await?;
                }
                &"I" => {
                    // skip
                }
                _ => Err(anyhow::anyhow!("Error unknown operation found"))?,
            }
        }

        Ok(Json::Null)
    }
}
