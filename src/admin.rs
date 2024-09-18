use crate::{create_cond_from_json, json_overwrite_key, list_query_to_list_param};

use super::{templates, AdminField, Json, ModelAdminTrait, Result};
use askama::DynTemplate;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    ops::Deref,
};

fn to_query_string(params: &HashMap<String, Vec<String>>, page: u64) -> String {
    let mut query = String::new();
    for (key, values) in params.iter() {
        if key == "_p" {
            continue;
        }
        for value in values {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(&format!("{}={}", key, value));
        }
    }
    if !query.is_empty() {
        query.push('&');
    }
    query.push_str(&format!("{}={}", "_p", page));
    query
}

trait Connector {
    fn get_connection(&self) -> &DatabaseConnection;
}

struct ConnectorImpl<C>
where
    C: Deref<Target = DatabaseConnection> + Sync + Send,
{
    conn: C,
}

impl<C> Connector for ConnectorImpl<C>
where
    C: Deref<Target = DatabaseConnection> + Sync + Send,
{
    fn get_connection(&self) -> &DatabaseConnection {
        self.conn.deref()
    }
}

enum FormType {
    CREATE,
    UPDATE,
    DELETE,
}

pub struct Admin {
    conn: Box<dyn Connector + Sync + Send>,
    pub models: HashMap<String, Box<dyn ModelAdminTrait + Send + Sync>>,
    pub site: templates::AdminSite,
}

impl Admin {
    pub fn new<C>(conn: C, sub_path: &str) -> Self
    where
        C: Deref<Target = DatabaseConnection> + Sync + Send + 'static,
    {
        Admin {
            conn: Box::new(ConnectorImpl { conn }),
            models: HashMap::new(),
            site: templates::AdminSite {
                title: "Admin".into(),
                models: Vec::new(),
                sub_path: sub_path.trim_end_matches('/').to_string(),
            },
        }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        self.conn.get_connection()
    }

    pub fn sub_path(&self) -> &str {
        &self.site.sub_path
    }

    pub fn add_model<T>(&mut self, model_admin: T) -> &Self
    where
        T: ModelAdminTrait + Send + Sync + 'static,
    {
        let table_name: String = model_admin.get_table_name().into();
        self.models
            .insert(table_name.clone(), Box::new(model_admin));
        self.site.models.push(table_name);
        self
    }

    pub fn get_model(&self, table_name: &str) -> Option<&Box<dyn ModelAdminTrait + Send + Sync>> {
        self.models.get(table_name)
    }

    pub async fn get_list_as_json(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        query_param: &HashMap<String, Vec<String>>,
    ) -> Result<Json> {
        let query = super::parse_query(query_param, model.get_list_per_page())?;
        let param = list_query_to_list_param(&query, &model.get_columns())?;
        let (total, object_list) = model.list(&self.get_connection(), &param).await?;
        super::json_convert_vec_to_json(model, total, object_list)
    }

    pub async fn get_list_template(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        query_param: &HashMap<String, Vec<String>>,
    ) -> Result<templates::AdminList> {
        let query = super::parse_query(query_param, model.get_list_per_page())?;
        let param = list_query_to_list_param(&query, &model.get_columns())?;
        let (count, object_list) = model.list(&self.get_connection(), &param).await?;
        let list_per_page = model.get_list_per_page();
        let num_pages = (count + list_per_page - 1) / list_per_page;
        let current_page = query.offset / list_per_page;
        let min_page = std::cmp::max(current_page as i64 - 3, 0) as u64;
        let max_page = std::cmp::min(current_page + 3, num_pages);

        let mut pages: Vec<templates::AdminListPage> = Vec::new();
        let blank = templates::AdminListPage {
            is_active: false,
            label: "...".to_string(),
            link: None,
        };
        let get_page = |p| -> templates::AdminListPage {
            templates::AdminListPage {
                is_active: p == current_page,
                label: format!("{}", p),
                link: Some(format!("?{}", to_query_string(query_param, p))),
            }
        };
        if min_page != 0 {
            pages.push(get_page(0));
        }
        if min_page > 1 {
            pages.push(blank.clone());
        }
        (min_page..max_page).for_each(|p| pages.push(get_page(p)));
        if num_pages > 0 && max_page < num_pages - 1 {
            pages.push(blank.clone());
        }
        if max_page < num_pages {
            pages.push(get_page(num_pages - 1));
        }

        let keys = model.list_display();
        Ok(templates::AdminList {
            site: self.site.clone(),
            model_name: model.get_table_name().into(),
            keys: keys.iter().cloned().collect(),
            rows: object_list
                .iter()
                .map(|x| {
                    Ok((
                        format!("update/{}/", model.json_to_key(x)?),
                        keys.iter()
                            .map(|key| super::json_force_str(x.get(key).unwrap()))
                            .collect(),
                    ))
                })
                .collect::<Result<Vec<_>>>()?,
            query: query.clone(),
            pages: pages,
            total: count,
        })
    }

    async fn get_form_fields(
        &self,
        base_fields: Vec<AdminField>,
        row: Option<&Json>,
        primary_keys: Vec<String>,
        form_type: FormType,
    ) -> Result<Vec<Box<dyn DynTemplate + Send>>> {
        let mut templates = Vec::new();
        let primary_keys: HashSet<String> = primary_keys.into_iter().collect();
        for field in base_fields.iter() {
            let disabled = match form_type {
                FormType::CREATE => match field {
                    AdminField::Field(f) => {
                        if f.fields().into_iter().any(|x| primary_keys.contains(&x)) {
                            continue;
                        }
                        false
                    }
                    _ => false,
                },
                FormType::UPDATE => match field {
                    AdminField::Field(f) => {
                        f.fields().into_iter().any(|x| primary_keys.contains(&x))
                    }
                    _ => false,
                },
                FormType::DELETE => true,
            };
            let r = field.get_template(self, row, "", disabled).await?;
            templates.push(r);
        }
        Ok(templates)
    }

    pub async fn get_create_template(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
    ) -> Result<templates::AdminCreateForm> {
        Ok(templates::AdminCreateForm {
            site: self.site.clone(),
            form_id: format!("{}-create", model.get_table_name()),
            page_id: "create".into(),
            model_name: model.get_table_name().into(),
            action: None,
            method: "POST".into(),
            fields: self
                .get_form_fields(
                    model.get_form_fields(),
                    None,
                    model.get_primary_keys(),
                    FormType::CREATE,
                )
                .await?,
        })
    }

    pub async fn get_update_template(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        row: &Json,
    ) -> Result<templates::AdminUpdateForm> {
        let id = model.json_to_key(row)?;

        Ok(templates::AdminUpdateForm {
            site: self.site.clone(),
            form_id: format!("{}-update", model.get_table_name()),
            page_id: id,
            model_name: model.get_table_name().into(),
            action: None,
            method: "POST".into(),
            fields: self
                .get_form_fields(
                    model.get_form_fields(),
                    Some(row),
                    model.get_primary_keys(),
                    FormType::UPDATE,
                )
                .await?,
        })
    }

    async fn handle_relation(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        data: &Json,
        txn: &DatabaseTransaction,
    ) -> Result<()> {
        for field in model.get_form_fields() {
            match field {
                AdminField::Relation(rel) => rel.commit(self, data, txn).await?,
                _ => Json::Null,
            };
        }
        Ok(())
    }

    pub async fn create(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        data: &Json,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Json> {
        let internal_txn = if txn.is_none() {
            Some(self.conn.get_connection().begin().await?)
        } else {
            None
        };

        let cur_txn = if txn.is_some() {
            txn.unwrap()
        } else {
            internal_txn.as_ref().unwrap()
        };

        // let txn = self.get_connection().begin().await?;

        let r = model.insert(&cur_txn, data).await?;
        let data = json_overwrite_key(data, &r)?;

        self.handle_relation(model, &data, cur_txn).await?;

        if let Some(txn_data) = internal_txn {
            txn_data.commit().await?;
        }

        Ok(data)
    }

    pub async fn update(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        data: &Json,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Json> {
        let internal_txn = if txn.is_none() {
            Some(self.conn.get_connection().begin().await?)
        } else {
            None
        };

        let cur_txn = if txn.is_some() {
            txn.unwrap()
        } else {
            internal_txn.as_ref().unwrap()
        };

        let r = model.update(cur_txn, data).await?;
        let data = json_overwrite_key(data, &r)?;

        self.handle_relation(model, &data, cur_txn).await?;

        if let Some(txn_data) = internal_txn {
            txn_data.commit().await?;
        }

        Ok(data)
    }

    pub async fn delete(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        data: &Json,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<u64> {
        let internal_txn = if txn.is_none() {
            Some(self.conn.get_connection().begin().await?)
        } else {
            None
        };
        let cur_txn = if txn.is_some() {
            txn.unwrap()
        } else {
            internal_txn.as_ref().unwrap()
        };
        let cond = create_cond_from_json(&model.get_primary_keys(), &data, true)?;
        let resp = model.delete(cur_txn, &cond).await?;
        if let Some(txn_data) = internal_txn {
            txn_data.commit().await?;
        }
        Ok(resp)
    }

    pub async fn get_delete_template(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        row: &Json,
    ) -> Result<templates::AdminDeleteForm> {
        let id = model.json_to_key(row)?;
        Ok(templates::AdminDeleteForm {
            site: self.site.clone(),
            form_id: format!("{}-delete", model.get_table_name()),
            page_id: id,
            model_name: model.get_table_name().into(),
            action: None,
            method: "POST".into(),
            fields: self
                .get_form_fields(
                    model.get_form_fields(),
                    Some(row),
                    model.get_primary_keys(),
                    FormType::DELETE,
                )
                .await?,
        })
    }
}

#[derive(Default)]
pub struct AdminBuilder {
    models: Vec<Box<dyn ModelAdminTrait + Send + Sync>>,
}

impl AdminBuilder {
    pub fn add_model<T>(mut self, model_admin: T) -> Self
    where
        T: ModelAdminTrait + Send + Sync + 'static,
    {
        self.models.push(Box::new(model_admin));
        self
    }

    pub fn build<C>(self, conn: C, sub_path: &str) -> Result<Admin>
    where
        C: Deref<Target = DatabaseConnection> + Sync + Send + 'static,
    {
        let mut models = HashMap::new();
        let mut site = templates::AdminSite {
            title: "Admin".into(),
            models: Vec::new(),
            sub_path: sub_path.trim_end_matches('/').to_string(),
        };
        let mut tables = HashSet::new();
        let mut related_tables = HashSet::new();
        for model_admin in self.models {
            let table_name: String = model_admin.get_table_name().into();
            tables.insert(table_name.clone());
            for form_field in model_admin.get_form_fields() {
                match form_field {
                    AdminField::Relation(r) => {
                        related_tables.extend(r.related_tables()?.into_iter());
                    }
                    _ => {}
                }
            }

            site.models.push(table_name.clone());
            models.insert(table_name.clone(), model_admin);
        }

        let shortage: Vec<String> = related_tables.difference(&tables).cloned().collect();
        anyhow::ensure!(
            shortage.is_empty(),
            "some tables are not found {shortage:?}"
        );

        Ok(Admin {
            conn: Box::new(ConnectorImpl { conn }),
            models,
            site,
        })
    }
}
