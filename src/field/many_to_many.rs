use super::{foreign_key_field::extract_table_name, AdminField, RelationTrait};
use crate::field::foreign_key_field::identity_to_vec_string;
use crate::templates::{
    AdminFormAutoComplete, AdminFormAutoCompleteChoice, AdminFormAutoCompleteCol,
};
use crate::{
    create_cond_from_input_json, create_cond_from_json, extract_cols_from_relation_def,
    json_force_str, ListParam, ModelAdminTrait,
};
use crate::{Admin, Json, Result};
use askama::DynTemplate;
use async_trait::async_trait;
use sea_orm::{Condition, DatabaseConnection, DatabaseTransaction,  RelationDef};

pub fn m2m_field(name: &str, from_def: RelationDef, to_def: RelationDef) -> AdminField {
    AdminField::Relation(Box::new(ManyToMany::new(name, from_def, to_def)))
}

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

    fn get_key(
        &self,
        parent_object: &serde_json::Map<String, Json>,
    ) -> Result<serde_json::Map<String, Json>> {
        Ok(identity_to_vec_string(&self.from_def.to_col)
            .into_iter()
            .zip(identity_to_vec_string(&self.from_def.from_col).into_iter())
            .map(|(to, fr)| (fr, parent_object.get(&to)))
            .filter(|x| x.1.is_some())
            .map(|x| (x.0, x.1.unwrap().clone()))
            .collect())
    }

    async fn list_related(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        conn: &DatabaseConnection,
        parent_object: &serde_json::Map<String, Json>,
    ) -> Result<Vec<Json>> {
        let key: serde_json::Map<String, Json> = self.get_key(parent_object)?;
        let cond = create_cond_from_json(
            &model.get_columns().into_iter().map(|x| x.0).collect(),
            &Json::Object(key.clone()),
            false,
        )?;

        let (_, cur_list) = model
            .list(
                conn,
                &crate::ListParam {
                    cond: cond,
                    ordering: vec![],
                    offset: None,
                    limit: None,
                },
            )
            .await?;
        Ok(cur_list)
    }
}

fn get_list_from_db_json(value: &Json, fcol: &Vec<String>) -> Result<Vec<String>> {
    let v = fcol
        .iter()
        .map(|key| value.get(key).ok_or(anyhow::anyhow!("key not found")))
        .collect::<Result<Vec<_>>>()?;
    Ok(v.into_iter().map(|x| json_force_str(x)).collect())
}

fn get_list_from_input_json(
    value: &serde_json::Map<String, Json>,
    key: &str,
) -> Result<Vec<String>> {
    let v = value.get(key).ok_or(anyhow::anyhow!("key not found"))?;
    let v = v.as_str().ok_or(anyhow::anyhow!("value is not str"))?;
    Ok(v.split(",").map(|x| x.to_string()).filter(|x| x.len() > 0).collect())
}

fn is_equal_vec(vl: &Vec<String>, vr: &Vec<String>) -> bool {
    if vl.len() != vr.len() {
        return false;
    }
    vl.iter().zip(vr.iter()).filter(|(x, y)| x == y).count() == vl.len()
}

fn check_exists(key: &Vec<String>, source: &Vec<Vec<String>>) -> bool {
    source.iter().filter(|x| is_equal_vec(key, x)).count() > 0
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
        let mut template = AdminFormAutoComplete {
            prefix: format!("{}{}-", prefix, self.name),
            name: format!("{}{}", prefix, self.name),
            label: self.name.clone(),
            choices: vec![],
            help_text: None,
            disabled: disabled,
            to_table: extract_table_name(&self.to_def.to_tbl)?,
            cols: extract_cols_from_relation_def(&self.to_def)?,
            nullable: true,
            multiple: true,
        };
        if let Some(parent_value) = parent_value {
            let parent_object = parent_value
                .as_object()
                .ok_or(anyhow::anyhow!("invalid json"))?;

            let model = admin
                .get_model(&extract_table_name(&self.from_def.from_tbl)?)
                .ok_or(anyhow::anyhow!("table not found"))?;

            let cur_list = self
                .list_related(model, admin.get_connection(), parent_object)
                .await?;

            let to_columns = identity_to_vec_string(&self.to_def.to_col);
            let mut cond = Condition::any();
            for row in cur_list.into_iter() {
                let mut filter: serde_json::Map<String, Json> = serde_json::Map::new();
                for (fr, to) in identity_to_vec_string(&self.to_def.from_col)
                    .into_iter()
                    .zip(identity_to_vec_string(&self.to_def.to_col).into_iter())
                {
                    filter.insert(
                        to,
                        row.get(&fr)
                            .ok_or(anyhow::anyhow!("key not found"))?
                            .clone(),
                    );
                }
                cond = cond.add(create_cond_from_json(
                    &to_columns,
                    &Json::Object(filter),
                    true,
                )?);
            }

            let to_model = admin
                .get_model(&extract_table_name(&self.to_def.to_tbl)?)
                .ok_or(anyhow::anyhow!("table not found"))?;

            let (_size, related) = if cond.is_empty() {
                (0, vec![])
            } else {to_model
                .list(
                    admin.get_connection(),
                    &ListParam {
                        cond,
                        ordering: vec![],
                        offset: None,
                        limit: None,
                    },
                )
                .await?
            };

            template.cols = template
                .cols
                .iter()
                .map(|x| AdminFormAutoCompleteCol {
                    value: related
                        .iter()
                        .map(|y| {
                            super::tool::get_value(Some(y), &x.to_col)
                                .map(|x| json_force_str(x))
                                .unwrap_or("".to_string())
                        })
                        .collect(),
                    from_col: x.from_col.clone(),
                    to_col: x.to_col.clone(),
                })
                .collect();

            template.choices = related
                .iter()
                .map(|x| {
                    Ok(AdminFormAutoCompleteChoice {
                        label: to_model.to_str(x)?,
                        value: to_model.json_to_key(x)?,
                        json_str: serde_json::to_string(x)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            println!("{:?}", related);
        }

        println!("fr:{:?} to:{:?}", self.from_def, self.to_def);
        Ok(Box::new(template))
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

        let model = admin
            .get_model(&extract_table_name(&self.from_def.from_tbl)?)
            .ok_or(anyhow::anyhow!("table not found"))?;

        let cur_list = self
            .list_related(model, admin.get_connection(), parent_object)
            .await?;

        // let key: serde_json::Map<String, Json> = identity_to_vec_string(&self.from_def.to_col)
        //     .into_iter()
        //     .zip(identity_to_vec_string(&self.from_def.from_col).into_iter())
        //     .map(|(to, fr)| (fr, parent_object.get(&to)))
        //     .filter(|x| x.1.is_some())
        //     .map(|x| (x.0, x.1.unwrap().clone()))
        //     .collect();
        // let cond = create_cond_from_json(
        //     &model.get_columns().into_iter().map(|x| x.0).collect(),
        //     &Json::Object(key.clone()),
        //     false,
        // )?;

        // let (_, cur_list) = model
        //     .list(
        //         admin.get_connection(),
        //         &crate::ListParam {
        //             cond: cond,
        //             ordering: vec![],
        //             offset: None,
        //             limit: None,
        //         },
        //     )
        //     .await?;
        let fcol = identity_to_vec_string(&self.to_def.from_col);
        let exist: Vec<Vec<String>> = cur_list
            .into_iter()
            .map(|x| get_list_from_db_json(&x, &fcol))
            .collect::<Result<Vec<_>>>()?;

        let input: Vec<Vec<String>> = identity_to_vec_string(&self.to_def.from_col)
            .into_iter()
            .map(|x| get_list_from_input_json(parent_object, &format!("{}-{}", self.name, x)))
            .collect::<Result<Vec<_>>>()?;
        let data_size = input[0].len();
        if input.iter().filter(|x| x.len() != data_size).count() > 0 {
            Err(anyhow::anyhow!("invalid input. mismatched data"))?
        }
        let input: Vec<Vec<String>> = (0..data_size)
            .map(|i| input.iter().map(|x| x[i].clone()).collect())
            .collect();

        let key: serde_json::Map<String, Json> = self.get_key(parent_object)?;
        for i in input.iter().filter(|x| !check_exists(x, &exist)) {
            let mut value = key.clone();
            i.into_iter()
                .zip(identity_to_vec_string(&self.to_def.from_col).into_iter())
                .for_each(|(v, k)| {
                    value.insert(k, Json::String(v.clone()));
                });
            let value = Json::Object(value);
            // println!("inserting {:?}", value);
            model.insert(txn, &value).await?;
        }

        let through_columns: Vec<_> = model.get_columns();
        for i in exist.iter().filter(|x| !check_exists(x, &input)) {
            let mut value = key.clone();
            i.into_iter()
                .zip(identity_to_vec_string(&self.to_def.from_col).into_iter())
                .for_each(|(v, k)| {
                    value.insert(k, Json::String(v.clone()));
                });
            let value = Json::Object(value);
            let cond = create_cond_from_input_json(&through_columns, &value, false)?;
            // println!("deleting {:?}", value);
            model.delete(txn, &cond).await?;
        }
        Ok(Json::Null)
    }
}
