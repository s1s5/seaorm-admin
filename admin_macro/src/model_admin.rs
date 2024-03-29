use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Attribute, ExprPath, Ident, Lit, Meta};

use crate::parse::parse_module;
use crate::parse::IdentOrLiteral;

type Result = std::result::Result<TokenStream, syn::Error>;

pub struct ModelAdminExpander {
    module: ExprPath,
    ident: Ident,
    list_display: Option<Vec<IdentOrLiteral>>,
    editable_fields: Option<Vec<syn::Expr>>,
    auto_complete: Option<Vec<syn::Ident>>,
    ordering: Option<Vec<(syn::Expr, syn::Expr)>>,
    search_fields: Option<Vec<syn::Expr>>,
    format: Option<Ident>,
    initial_value: Option<Ident>,
    form_fields: Option<Vec<syn::Expr>>,
}

impl ModelAdminExpander {
    pub fn new(ident: Ident, attrs: Vec<Attribute>) -> std::result::Result<Self, syn::Error> {
        let mut module = None;
        let mut list_display = None;
        let mut editable_fields = None;
        let mut auto_complete = None;
        let mut ordering = None;
        let mut search_fields = None;
        let mut format = None;
        let mut initial_value = None;
        let mut form_fields = None;

        attrs.iter().try_for_each(|attr| {
            if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                for meta in list.iter() {
                    if let Meta::NameValue(nv) = meta {
                        if let Some(ident) = nv.path.get_ident() {
                            if ident == "module" {
                                module = Some(parse_module(ident, nv)?.clone());
                            } else if ident == "list_display" {
                                list_display = Some(super::parse::parse_list_display(ident, &nv)?);
                            } else if ident == "fields" {
                                editable_fields =
                                    Some(super::parse::parse_editable_fields(ident, &nv)?);
                            } else if ident == "auto_complete" {
                                auto_complete =
                                    Some(super::parse::parse_auto_complete(ident, &nv)?);
                            } else if ident == "ordering" {
                                ordering = Some(super::parse::parse_ordering(ident, nv)?);
                            } else if ident == "search_fields" {
                                search_fields =
                                    Some(super::parse::parse_search_fields(ident, &nv)?);
                            } else if ident == "format" {
                                format = Some(super::parse::parse_format(ident, nv)?.clone());
                            } else if ident == "initial_value" {
                                initial_value =
                                    Some(super::parse::parse_initial_value(ident, nv)?.clone());
                            } else if ident == "form_fields" {
                                form_fields =
                                    Some(super::parse::parse_form_fields(ident, nv)?.clone());
                            }
                        }
                    }
                }
            }
            Ok::<(), syn::Error>(())
        })?;
        let module = module.ok_or(syn::Error::new(ident.span(), "module must be specified"))?;

        Ok(ModelAdminExpander {
            module,
            ident,
            list_display,
            editable_fields,
            auto_complete,
            ordering,
            search_fields,
            format,
            initial_value,
            form_fields,
        })
    }

    fn expand_get_list_display(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        if let Some(list_display) = self.list_display.clone() {
            let list_display: Vec<_> = list_display
                .iter()
                .map(|e| match e {
                    IdentOrLiteral::Path(i) => {
                        quote!(#module::Column::#i . to_string())
                    }
                    IdentOrLiteral::Literal(s) => {
                        quote!(#s . to_string())
                    }
                })
                .collect();
            Ok(quote! {
                impl #ident {
                    fn get_list_display() -> Vec<String> {
                        use seaorm_admin::sea_orm::Iden;
                        vec![#(#list_display),*]
                    }
                }
            })
        } else {
            Ok(quote! {
                impl #ident {
                    fn get_list_display() -> Vec<String> {
                        use seaorm_admin::sea_orm::{Iden, Iterable};
                        #module :: Column::iter().map(|x| x.to_string()).collect()
                    }
                }
            })
        }
    }

    fn expand_get_auto_complete(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        if let Some(auto_complete) = self.auto_complete.clone() {
            Ok(quote! {
                impl #ident {
                    fn get_auto_complete() -> Vec<#module::Relation> {
                        vec![#(#module :: Relation:: #auto_complete),*]
                    }
                }
            })
        } else {
            Ok(quote! {
                impl #ident {
                    fn get_auto_complete() -> Vec<#module::Relation> {
                        vec![]
                    }
                }
            })
        }
    }

    fn expand_get_ordering(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        if let Some(ordering) = self.ordering.clone() {
            let (columns, order): (Vec<syn::Expr>, Vec<syn::Expr>) = ordering.into_iter().unzip();

            Ok(quote! {
                impl #ident {
                    fn get_ordering() -> Vec<(#module::Column, seaorm_admin::sea_orm::Order)> {
                        vec![#((#module :: Column :: #columns, seaorm_admin::sea_orm::Order:: #order)),*]
                    }
                }
            })
        } else {
            Ok(quote! {
                impl #ident {
                    fn get_ordering() -> Vec<(#module::Column, seaorm_admin::sea_orm::Order)> {
                        use seaorm_admin::sea_orm::{Iterable, PrimaryKeyToColumn};
                        #module::PrimaryKey::iter()
                            .map(|x| (x.into_column(), seaorm_admin::sea_orm::Order::Desc))
                            .collect()
                    }
                }
            })
        }
    }

    fn expand_get_fields(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        Ok(quote!(
            impl #ident {
                fn get_fields() -> Vec<#module :: Column> {
                    use seaorm_admin::sea_orm::Iterable;
                    #module :: Column::iter().collect()
                }
            }
        ))
    }

    fn expand_get_editable_fields(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        if let Some(form_fields) = &self.editable_fields {
            Ok(quote! {
                impl #ident {
                    fn get_editable_fields() -> Vec<#module::Column> {
                        vec![#(#module :: Column:: #form_fields),*]
                    }
                }
            })
        } else {
            Ok(quote!(
                impl #ident {
                    fn get_editable_fields() -> Vec<#module :: Column> {
                        use seaorm_admin::sea_orm::Iterable;
                        #module :: Column::iter().collect()
                    }
                }
            ))
        }
    }

    fn expand_get_keys(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        Ok(quote!(
            impl #ident {
                fn get_keys() -> Vec<#module :: Column> {
                    use seaorm_admin::sea_orm::{Iterable, PrimaryKeyToColumn};
                    #module :: PrimaryKey::iter().map(|x| x.into_column()).collect()
                }
            }
        ))
    }

    fn expand_get_search_fields(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        if let Some(search_fields) = &self.search_fields {
            Ok(quote!(
                impl #ident {
                    fn get_search_fields() -> Vec<#module :: Column> {
                        vec![#(#module :: Column:: #search_fields),*]
                    }
                }
            ))
        } else {
            Ok(quote!(
                impl #ident {
                    fn get_search_fields() -> Vec<#module :: Column> {
                        use seaorm_admin::sea_orm::Iterable;
                        #module::Column::iter().collect()
                    }
                }
            ))
        }
    }

    fn expand_get_list_per_page(&self) -> Result {
        let ident = &self.ident;
        Ok(quote!(
            impl #ident {
                fn get_list_per_page() -> u64 {
                    50
                }
            }
        ))
    }

    fn expand_get_initial_value(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        if let Some(initial_value) = &self.initial_value {
            Ok(quote!(
                impl #ident {
                    fn get_initial_value() -> #module::ActiveModel {
                        #initial_value ()
                    }
                }
            ))
        } else {
            Ok(quote!(
                impl #ident {
                    fn get_initial_value() -> #module::ActiveModel {
                        #module::ActiveModel { ..Default::default() }
                    }
                }
            ))
        }
    }

    fn expand_impl(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;

        Ok(quote!(
            #[seaorm_admin::async_trait]
            impl seaorm_admin::ModelAdminTrait for #ident {
                fn get_table_name(&self) -> &str {
                    use seaorm_admin::sea_orm::EntityName;
                    #module ::Entity{}.table_name()
                }

                fn get_list_per_page(&self) -> u64 {
                    #ident::get_list_per_page()
                }

                fn get_columns(&self) -> Vec<(String, seaorm_admin::sea_orm::ColumnDef)> {
                    use seaorm_admin::sea_orm::{Iden, Iterable, ColumnTrait};
                    #module::Column::iter().map(|x| (x.to_string(), x.def())).collect()
                }

                fn get_primary_keys(&self) -> Vec<String> {
                    use seaorm_admin::sea_orm::Iden;
                    #ident::get_keys().into_iter().map(|x| x.to_string()).collect()
                }

                fn to_str(&self, value: &seaorm_admin::Json) -> seaorm_admin::Result<String> {
                    #ident::to_str_impl(value)
                }

                fn json_to_key(&self, value: &seaorm_admin::Json) -> seaorm_admin::Result<String> {
                    seaorm_admin::to_key_string(&#ident::get_keys(), value)
                }

                fn key_to_json(&self, key: &str) -> seaorm_admin::Result<seaorm_admin::Json> {
                    seaorm_admin::from_key_string(&#ident::get_keys(), key)
                }

                fn list_display(&self) -> Vec<String> {
                    #ident::get_list_display()
                }

                fn get_form_fields(&self) -> Vec<seaorm_admin::AdminField> {
                    #ident::get_form_fields_impl()
                }

                async fn list(
                    &self,
                    conn: &seaorm_admin::sea_orm::DatabaseConnection,
                    param: &seaorm_admin::ListParam,
                ) -> seaorm_admin::Result<(u64, Vec<seaorm_admin::Json>)> {
                    #ident::list_impl(conn, param).await
                }

                async fn get(&self, conn: &seaorm_admin::sea_orm::DatabaseConnection, cond: &seaorm_admin::sea_orm::Condition) -> seaorm_admin::Result<Option<seaorm_admin::Json>> {
                    #ident::get_impl(conn, cond).await
                }

                async fn insert(&self, conn: &seaorm_admin::sea_orm::DatabaseTransaction, value: &seaorm_admin::Json) -> seaorm_admin::Result<seaorm_admin::Json> {
                    #ident::insert_impl(conn, value).await
                }

                async fn update(&self, conn: &seaorm_admin::sea_orm::DatabaseTransaction, value: &seaorm_admin::Json) -> seaorm_admin::Result<seaorm_admin::Json> {
                    #ident::update_impl(conn, value).await
                }

                async fn delete(&self, conn: &seaorm_admin::sea_orm::DatabaseTransaction, value: &seaorm_admin::sea_orm::Condition) -> seaorm_admin::Result<u64> {
                    #ident::delete_impl(conn, value).await
                }
            }
        ))
    }

    fn expand_to_str_impl(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        if let Some(format) = &self.format {
            Ok(quote!(
                impl #ident {
                    fn to_str_impl(value: &seaorm_admin::Json) -> seaorm_admin::Result<String> {
                        use seaorm_admin::sea_orm::TryIntoModel;
                        let mut model = #module::ActiveModel { ..Default::default() };
                        seaorm_admin::set_from_json(&mut model, &#ident::get_fields(), &value)?;
                        Ok(#format(&model.try_into_model()?))
                    }
                }
            ))
        } else {
            Ok(quote!(
                impl #ident {
                    fn to_str_impl(value: &seaorm_admin::Json) -> seaorm_admin::Result<String> {
                        use seaorm_admin::sea_orm::{EntityName, Iterable, PrimaryKeyToColumn, Iden};
                        let n = seaorm_admin::Json::Null;
                        let keys: Vec<_> = #ident::get_keys().iter().map(|x| x.to_string()).collect();
                        let values: Vec<_> = keys.iter().map(|x| value.get(x).unwrap_or(&n)).collect();
                        let values: Vec<_> = values.iter().map(|x| seaorm_admin::json_force_str(x)).collect();
                        let s: Vec<_> = keys.into_iter()
                                            .zip(values.into_iter())
                                            .map(|(x, y) | format!("{}={}", x, y))
                                            .collect();
                        Ok(format!("{}: [{}]", #module ::Entity{}.table_name(), s.join(", ")))
                    }
                }
            ))
        }
    }

    fn expand_to_json_for_list(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        let extra_fields: Vec<_> = if let Some(list_display) = self.list_display.clone() {
            list_display
                .iter()
                .map(|e| match e {
                    IdentOrLiteral::Path(_i) => None,
                    IdentOrLiteral::Literal(s) => {
                        let id = match &s.lit {
                            Lit::Str(s) => s.value(),
                            _ => panic!("Unexpected ExprLit type found"),
                        };
                        let id = format_ident!("{}", id);
                        Some(quote!(jv[#s] = #id(&x).into()))
                    }
                })
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect()
        } else {
            vec![]
        };

        if extra_fields.is_empty() {
            Ok(quote! {
                impl #ident {
                    #[inline]
                    fn convert_to_json_for_list(x: #module::Model, fields: &Vec<#module :: Column>) -> seaorm_admin::Result<seaorm_admin::Json> {
                        seaorm_admin::to_json(&x, &fields)
                    }
                }
            })
        } else {
            Ok(quote! {
                impl #ident {
                    fn convert_to_json_for_list(x: #module::Model, fields: &Vec<#module :: Column>) -> seaorm_admin::Result<seaorm_admin::Json> {
                        let mut jv = seaorm_admin::to_json(&x, &fields)?;
                        #(#extra_fields);*;
                        Ok(jv)
                    }
                }
            })
        }
    }

    fn expand_get_form_fields_impl(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        let auto_complete = self.auto_complete.clone().unwrap_or(vec![]);
        let form_fields = self.form_fields.clone().unwrap_or(vec![]);

        Ok(quote!(
        impl #ident {
            fn get_form_fields_impl() -> Vec<seaorm_admin::AdminField> {
                use std::collections::{HashSet, HashMap};
                use seaorm_admin::sea_orm::{Iden, Iterable, PrimaryKeyToColumn, ActiveModelTrait, ColumnTrait, RelationTrait};
                let auto_complete: Vec<sea_orm::RelationDef> = vec![#((#module::Relation::#auto_complete.def())),*];
                let mut ac_col_set:HashSet<String> = auto_complete
                    .iter()
                    .map(|x| seaorm_admin::extract_cols_from_relation_def(&x).unwrap())
                    .flatten()
                    .map(|x| x.from_col)
                    .collect();
                let ac_fields: Vec<_> = auto_complete
                    .into_iter()
                    .map(|x| seaorm_admin::ForeignKeyField::new(
                        &x,
                        seaorm_admin::relation_def_is_nullable(
                            &x,
                            &#module :: Column::iter().collect(),
                        ),
                    ))
                    .filter(|x| x.is_ok())
                    .map(|x| x.unwrap())
                    .map(|x| seaorm_admin::AdminField::Field(Box::new(x)))
                    .collect();
                let mut ex_fields: Vec<seaorm_admin::AdminField> = vec![#(#form_fields),*];
                ac_col_set.extend(ex_fields
                    .iter()
                    .map(|x| {
                        match x {
                            seaorm_admin::AdminField::Field(f) => f.fields(),
                            _ => vec![],
                        }
                    }).flatten());

                let mut fields: Vec<seaorm_admin::AdminField> = #ident::get_editable_fields()
                    .into_iter()
                    .filter(|x| !ac_col_set.contains(&x.to_string()))
                    .filter(|x| !ac_col_set.contains(&x.to_string()))
                    .map(|x| {
                        seaorm_admin::get_default_field(&x.to_string(), &x.def().get_column_type())
                    })
                    .filter(|x| x.is_ok()).map(|x| x.unwrap()).collect();
                fields.extend(ac_fields);
                fields.extend(ex_fields);
                fields
            }
        }))
    }

    fn expand_list_impl(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;

        Ok(quote!(
            impl #ident {
            async fn list_impl(
                conn: &seaorm_admin::sea_orm::DatabaseConnection,
                param: &seaorm_admin::ListParam,
            ) -> seaorm_admin::Result<(u64, Vec<seaorm_admin::Json>)> {
                use seaorm_admin::sea_orm::{EntityTrait, QuerySelect, PaginatorTrait, QueryFilter};

                let fields = #ident::get_fields();
                let qs = #module::Entity::find();
                let qs = if param.ordering.len() > 0 {
                    seaorm_admin::set_ordering_from_query(qs, &param.ordering, &#ident::get_fields())?
                } else {
                    seaorm_admin::set_ordering(qs, &#ident::get_ordering())?
                };

                let qs = if param.cond.is_empty() {
                    qs
                } else {
                    qs.filter(param.cond.clone())
                };
                let qs = if let Some(offset) = param.offset { qs.offset(offset) } else { qs };
                let qs = if let Some(limit) = param.limit { qs.limit(limit) } else { qs };
                let count = qs.clone().count(conn).await?;
                qs.all(conn).await?
                    .into_iter()
                    .map(|x| #ident::convert_to_json_for_list(x, &fields))
                    .collect::<seaorm_admin::Result<Vec<_>>>()
                    .map(|x| (count, x))
            }
        }
        ))
    }

    fn expand_get_impl(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;

        Ok(quote!(
            impl #ident {
            async fn get_impl(
                conn: &seaorm_admin::sea_orm::DatabaseConnection,
                cond: &seaorm_admin::sea_orm::Condition
            ) -> seaorm_admin::Result<Option<seaorm_admin::Json>> {
                use seaorm_admin::sea_orm::{EntityTrait, QueryFilter};

                let fields = #ident::get_fields();
                let qs = #module::Entity::find();
                let qs = qs.filter(cond.clone());
                Ok(if let Some(model) = qs.one(conn).await? {
                    Some(seaorm_admin::to_json(&model, &fields)?)
                } else {
                    None
                })
            }
        }
        ))
    }

    fn expand_insert_impl(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;

        Ok(quote!(
            impl #ident {
                async fn insert_impl(
                    conn: &seaorm_admin::sea_orm::DatabaseTransaction,
                    value: &seaorm_admin::Json
                ) -> seaorm_admin::Result<seaorm_admin::Json> {
                    use seaorm_admin::sea_orm::{EntityTrait, ActiveModelTrait, TryIntoModel};

                    let fields = #ident::get_fields();
                    let mut model = #ident::get_initial_value();
                    seaorm_admin::set_from_json(&mut model, &fields, value)?;
                    let saved: #module::Model = model.insert(conn).await?.try_into_model()?;
                    seaorm_admin::to_json(&saved, &fields)
                }
            }
        ))
    }

    fn expand_update_impl(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;

        Ok(quote!(
            impl #ident {
                async fn update_impl(
                    conn: &seaorm_admin::sea_orm::DatabaseTransaction,
                    value: &seaorm_admin::Json
                ) -> seaorm_admin::Result<seaorm_admin::Json> {
                    use seaorm_admin::sea_orm::{TryIntoModel, ActiveModelTrait, EntityTrait};

                    let fields = #ident::get_fields();
                    let mut model = #module::ActiveModel { ..Default::default() };
                    seaorm_admin::set_from_json(&mut model, &fields, value)?;
                    let saved: #module::Model = model.save(conn).await?.try_into_model().unwrap();
                    seaorm_admin::to_json(&saved, &fields)
                }
            }
        ))
    }

    fn expand_delete_impl(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;

        Ok(quote!(
            impl #ident {
            async fn delete_impl(
                conn: &seaorm_admin::sea_orm::DatabaseTransaction,
                cond: &seaorm_admin::sea_orm::Condition,
            ) -> seaorm_admin::Result<u64> {
                use seaorm_admin::sea_orm::{EntityTrait, ModelTrait, QueryFilter};

                let qs = #module::Entity::find();
                let qs = qs.filter(cond.clone());
                Ok(if let Some(model) = qs.one(conn).await? {
                    model.delete(conn).await?.rows_affected
                } else {
                    0
                })

            }

        }
        ))
    }

    pub fn expand(&self) -> Result {
        Ok(TokenStream::from_iter([
            self.expand_get_list_display()?,
            self.expand_get_auto_complete()?,
            self.expand_get_ordering()?,
            self.expand_get_fields()?,
            self.expand_get_editable_fields()?,
            self.expand_get_keys()?,
            self.expand_get_search_fields()?,
            self.expand_get_list_per_page()?,
            self.expand_get_initial_value()?,
            self.expand_impl()?,
            self.expand_to_str_impl()?,
            self.expand_to_json_for_list()?,
            self.expand_get_form_fields_impl()?,
            self.expand_list_impl()?,
            self.expand_get_impl()?,
            self.expand_insert_impl()?,
            self.expand_update_impl()?,
            self.expand_delete_impl()?,
        ]))
    }
}
