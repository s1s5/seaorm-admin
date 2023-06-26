use super::{templates, AdminField, Json, ModelAdminTrait, Result};
use askama::DynTemplate;
use sea_orm::DatabaseConnection;
use std::{collections::HashMap, ops::Deref};

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
        let (total, object_list) = model.list(&self.get_connection(), &query).await?;
        super::json_convert_vec_to_json(model, total, object_list)
    }

    pub async fn get_list_template(
        &self,
        model: &Box<dyn ModelAdminTrait + Send + Sync>,
        query_param: &HashMap<String, Vec<String>>,
    ) -> Result<templates::AdminList> {
        let query = super::parse_query(query_param, model.get_list_per_page())?;
        let (count, object_list) = model.list(&self.get_connection(), &query).await?;
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
        form_type: FormType,
    ) -> Result<Vec<Box<dyn DynTemplate + Send>>> {
        let mut templates = Vec::new();
        for field in base_fields.iter() {
            let disabled = match form_type {
                FormType::CREATE => false,
                FormType::UPDATE => false,
                FormType::DELETE => true,
            };
            let r = field.get_template(self, row, disabled).await?;
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
                .get_form_fields(model.get_form_fields(), None, FormType::CREATE)
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
                .get_form_fields(model.get_form_fields(), Some(row), FormType::UPDATE)
                .await?,
        })
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
                .get_form_fields(model.get_form_fields(), Some(row), FormType::DELETE)
                .await?,
        })
    }
}
