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
    form_fields: Option<Vec<syn::Expr>>,
    auto_complete: Option<Vec<syn::Expr>>,
    ordering: Option<Vec<(syn::Expr, syn::Expr)>>,
    search_fields: Option<Vec<syn::Expr>>,
    format: Option<Ident>,
    initial_value: Option<Ident>,
}

impl ModelAdminExpander {
    pub fn new(ident: Ident, attrs: Vec<Attribute>) -> std::result::Result<Self, syn::Error> {
        let mut module = None;
        let mut list_display = None;
        let mut form_fields = None;
        let mut auto_complete = None;
        let mut ordering = None;
        let mut search_fields = None;
        let mut format = None;
        let mut initial_value = None;

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
                                form_fields = Some(super::parse::parse_form_fields(ident, &nv)?);
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
            form_fields,
            auto_complete,
            ordering,
            search_fields,
            format,
            initial_value,
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

    fn expand_get_form_fields(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;
        if let Some(form_fields) = &self.form_fields {
            Ok(quote! {
                impl #ident {
                    fn get_form_fields() -> Vec<#module::Column> {
                        vec![#(#module :: Column:: #form_fields),*]
                    }
                }
            })
        } else {
            Ok(quote!(
                impl #ident {
                    fn get_form_fields() -> Vec<#module :: Column> {
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
                        use seaorm_admin::sea_orm::{Iterable, PrimaryKeyToColumn};
                        #module :: PrimaryKey::iter().map(|x| x.into_column()).collect()
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

                fn to_str(&self, value: &seaorm_admin::Json) -> seaorm_admin::Result<String> {
                    #ident::to_str_impl(value)
                }

                fn json_to_key(&self, value: &seaorm_admin::Json) -> seaorm_admin::Result<String> {
                    seaorm_admin::to_key_string(&#ident::get_keys(), value)
                }

                fn key_to_json(&self, key: &str) -> seaorm_admin::Result<seaorm_admin::Json> {
                    seaorm_admin::from_key_string(&#ident::get_keys(), key)
                }

                fn get_create_form_fields(&self) -> Vec<seaorm_admin::AdminField> {
                    use seaorm_admin::sea_orm::{Iden, Iterable, PrimaryKeyToColumn, ActiveModelTrait};

                    // primary keyは隠す
                    let keys: std::collections::HashSet<_> = #module :: PrimaryKey::iter().map(|x| x.into_column().to_string()).collect();
                    let model = #ident::get_initial_value();

                    #ident::get_form_fields().into_iter().filter(
                        |x| !keys.contains(&x.to_string())).filter(
                            |x| !model.get(*x).is_set()
                        ).map(
                            |x| seaorm_admin::AdminField::create_from(&x, true)).collect()
                }

                fn get_update_form_fields(&self) -> Vec<seaorm_admin::AdminField> {
                    use seaorm_admin::sea_orm::{Iden, Iterable, PrimaryKeyToColumn};

                    let keys: std::collections::HashSet<_> = #module :: PrimaryKey::iter().map(|x| x.into_column().to_string()).collect();
                    #ident::get_form_fields()
                        .into_iter()
                        .map(|x| seaorm_admin::AdminField::create_from(&x, !keys.contains(&x.to_string())))
                        .collect()
                }

                fn list_display(&self) -> Vec<String> {
                    #ident::get_list_display()
                }

                fn get_auto_complete(&self) -> Vec<seaorm_admin::sea_orm::RelationDef> {
                    use seaorm_admin::sea_orm::RelationTrait;
                    #ident::get_auto_complete().iter().map(|x| x.def()).collect()
                }

                async fn list(
                    &self,
                    conn: &seaorm_admin::sea_orm::DatabaseConnection,
                    query: &seaorm_admin::ListQuery,
                ) -> seaorm_admin::Result<(u64, Vec<seaorm_admin::Json>)> {
                    #ident::list_impl(conn, query).await
                }

                async fn get(&self, conn: &seaorm_admin::sea_orm::DatabaseConnection, key: seaorm_admin::Json) -> seaorm_admin::Result<Option<seaorm_admin::Json>> {
                    #ident::get_impl(conn, key).await
                }

                async fn insert(&self, conn: &seaorm_admin::sea_orm::DatabaseConnection, value: seaorm_admin::Json) -> seaorm_admin::Result<seaorm_admin::Json> {
                    #ident::insert_impl(conn, value).await
                }

                async fn update(&self, conn: &seaorm_admin::sea_orm::DatabaseConnection, value: seaorm_admin::Json) -> seaorm_admin::Result<seaorm_admin::Json> {
                    #ident::update_impl(conn, value).await
                }

                async fn delete(&self, conn: &seaorm_admin::sea_orm::DatabaseConnection, value: seaorm_admin::Json) -> seaorm_admin::Result<u64> {
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

    fn expand_list_impl(&self) -> Result {
        let ident = &self.ident;
        let module = &self.module;

        Ok(quote!(
            impl #ident {
            async fn list_impl(
                conn: &seaorm_admin::sea_orm::DatabaseConnection,
                query: &seaorm_admin::ListQuery,
            ) -> seaorm_admin::Result<(u64, Vec<seaorm_admin::Json>)> {
                use seaorm_admin::sea_orm::{EntityTrait, QuerySelect, PaginatorTrait};

                let fields = #ident::get_fields();
                let qs = #module::Entity::find();
                let qs = if query.ordering.len() > 0 {
                    seaorm_admin::set_ordering_from_query(qs, &query.ordering, &#ident::get_fields())?
                } else {
                    seaorm_admin::set_ordering(qs, &#ident::get_ordering())?
                };
                let qs = seaorm_admin::filter_by_hash_map(qs, &fields, &query.filter)?;
                let qs = seaorm_admin::search_by_queries(qs, &#ident::get_search_fields(), &query.queries)?;
                let count = qs.clone().count(conn).await?;
                qs.offset(query.offset)
                    .limit(query.limit)
                    .all(conn).await?
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
                key: seaorm_admin::Json
            ) -> seaorm_admin::Result<Option<seaorm_admin::Json>> {
                use seaorm_admin::sea_orm::EntityTrait;

                let fields = #ident::get_fields();
                let qs = #module::Entity::find();
                let qs = {
                    let mut fm = #module::ActiveModel { ..Default::default() };
                    seaorm_admin::set_from_json(&mut fm, &fields, &key)?;
                    seaorm_admin::filter_by_columns(qs, &#ident::get_keys(), &fm, true)?
                };
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
                    conn: &seaorm_admin::sea_orm::DatabaseConnection,
                    value: seaorm_admin::Json
                ) -> seaorm_admin::Result<seaorm_admin::Json> {
                    use seaorm_admin::sea_orm::{EntityTrait, ActiveModelTrait, TryIntoModel};

                    let fields = #ident::get_fields();
                    let mut model = #ident::get_initial_value();
                    seaorm_admin::set_from_json(&mut model, &fields, &value)?;
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
                    conn: &seaorm_admin::sea_orm::DatabaseConnection,
                    value: seaorm_admin::Json
                ) -> seaorm_admin::Result<seaorm_admin::Json> {
                    use seaorm_admin::sea_orm::{TryIntoModel, ActiveModelTrait, EntityTrait};

                    let fields = #ident::get_fields();
                    let mut model = #module::ActiveModel { ..Default::default() };
                    seaorm_admin::set_from_json(&mut model, &fields, &value)?;
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
                conn: &seaorm_admin::sea_orm::DatabaseConnection,
                value: seaorm_admin::Json
            ) -> seaorm_admin::Result<u64> {
                use seaorm_admin::sea_orm::{EntityTrait, ModelTrait};

                let qs = #module::Entity::find();
                let qs = {
                    let mut fm = #module::ActiveModel { ..Default::default() };
                    seaorm_admin::set_from_json(&mut fm, &#ident::get_fields(), &value)?;
                    seaorm_admin::filter_by_columns(qs, &#ident::get_keys(), &fm, true)?
                };
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
            self.expand_get_form_fields()?,
            self.expand_get_keys()?,
            self.expand_get_search_fields()?,
            self.expand_get_list_per_page()?,
            self.expand_get_initial_value()?,
            self.expand_impl()?,
            self.expand_to_str_impl()?,
            self.expand_to_json_for_list()?,
            self.expand_list_impl()?,
            self.expand_get_impl()?,
            self.expand_insert_impl()?,
            self.expand_update_impl()?,
            self.expand_delete_impl()?,
        ]))
    }
}
