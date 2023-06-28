use crate::{Json, ListParam, ListQuery, Result};
use sea_orm::sea_query::{Alias, Condition, Expr, SeaRc};
use sea_orm::{ColumnDef, DynIden, EntityTrait, Iden, QueryOrder, Select};
use std::collections::HashMap;

// enum ConditionOp {
//     Eq,
//     // Like,
//     // Gt,
//     // Gte,
//     // Lt,
//     // Lte,
// }

// fn get_condition<C>(col: &C, op: &ConditionOp, value: &str) -> Condition
// where
//     C: ColumnTrait,
// {
//     match op {
//         ConditionOp::Eq => Expr::col(*col).eq(value).into_condition(),
//     }
// }
pub fn list_query_to_list_param(
    query: &ListQuery,
    columns: &Vec<(String, ColumnDef)>,
) -> Result<ListParam> {
    let mut cond = Condition::all();
    let c = create_cond_from_hash_map(
        &columns.iter().map(|x| x.0.clone()).collect(),
        &query.filter,
    )?;
    if !c.is_empty() {
        cond = cond.add(c);
    }
    let c = create_cond_from_search_queries(columns, &query.queries)?;
    if !c.is_empty() {
        cond = cond.add(c);
    }

    Ok(ListParam {
        cond: cond,
        ordering: query.ordering.clone(),
        offset: Some(query.offset),
        limit: Some(query.limit),
    })
}

pub fn set_ordering<E>(
    qs: Select<E>,
    ordering: &Vec<(<E as EntityTrait>::Column, sea_orm::Order)>,
) -> Result<Select<E>>
where
    E: EntityTrait,
{
    let mut qs = qs;
    for (col, ord) in ordering {
        qs = qs.order_by(*col, ord.clone());
    }
    Ok(qs)
}

pub fn set_ordering_from_query<E>(
    qs: Select<E>,
    ordering: &Vec<(String, sea_orm::Order)>,
    columns: &Vec<<E as EntityTrait>::Column>,
) -> Result<Select<E>>
where
    E: EntityTrait,
{
    let m: HashMap<_, _> = columns.iter().map(|x| (x.to_string(), x)).collect();
    let ordering: Vec<_> = ordering
        .iter()
        .filter(|x| m.contains_key(&x.0))
        .map(|x| (m[&x.0].clone(), x.1.clone()))
        .collect();
    set_ordering(qs, &ordering)
}

pub fn create_cond_from_hash_map(
    columns: &Vec<String>,
    filter: &HashMap<String, Vec<String>>,
) -> Result<Condition> {
    let mut cond = Condition::all();

    for col in columns.iter() {
        // TODO: support "like", "gt", etc..
        if let Some(queries) = filter.get(col) {
            let mut pcond = Condition::any();
            for value in queries {
                let col: DynIden = SeaRc::new(Alias::new(col));
                pcond = pcond.add(Expr::col(col).eq(value.clone()));
            }
            cond = cond.add(pcond);
        }
    }

    Ok(cond)
}

pub fn create_cond_from_json(
    columns: &Vec<String>,
    filter: &Json,
    check_exists: bool,
) -> Result<Condition> {
    let filter = filter
        .as_object()
        .ok_or(anyhow::anyhow!("filter must be object"))?;
    let mut cond = Condition::all();
    for col in columns.iter() {
        if let Some(value) = filter.get(col) {
            let col: DynIden = SeaRc::new(Alias::new(col));
            match value {
                Json::Null => {}
                Json::Bool(b) => {
                    cond = cond.add(Expr::col(col).eq(*b));
                }
                serde_json::Value::Number(n) => {
                    if n.is_f64() {
                        cond = cond.add(Expr::col(col).eq(n.as_f64()));
                    } else {
                        cond = cond.add(Expr::col(col).eq(n.as_i64()));
                    }
                }
                serde_json::Value::String(s) => {
                    cond = cond.add(Expr::col(col).eq(s));
                }
                _ => Err(anyhow::anyhow!("Unsupport value type"))?,
            }
        } else if check_exists {
            return Err(anyhow::anyhow!("key not found"));
        }
    }

    Ok(cond)
}

pub fn create_cond_from_search_queries(
    columns: &Vec<(String, ColumnDef)>,
    queries: &Vec<String>,
) -> Result<Condition> {
    let mut cond = Condition::any();
    for (col_name, col_def) in columns {
        let col: DynIden = SeaRc::new(Alias::new(col_name));
        match col_def.get_column_type() {
            sea_orm::ColumnType::Char(_)
            | sea_orm::ColumnType::String(_)
            | sea_orm::ColumnType::Text
            | sea_orm::ColumnType::Uuid => {
                let mut pcond = Condition::all();
                for value in queries {
                    pcond =
                        pcond.add(Expr::col(col.clone()).like(format!("%{}%", value.to_string())));
                }
                if !pcond.is_empty() {
                    cond = cond.add(pcond);
                }
            }
            sea_orm::ColumnType::TinyInteger
            | sea_orm::ColumnType::SmallInteger
            | sea_orm::ColumnType::Integer
            | sea_orm::ColumnType::BigInteger
            | sea_orm::ColumnType::TinyUnsigned
            | sea_orm::ColumnType::SmallUnsigned
            | sea_orm::ColumnType::Unsigned
            | sea_orm::ColumnType::BigUnsigned => {
                let mut pcond = Condition::all();
                for value in queries
                    .iter()
                    .map(|x| x.parse::<i64>().clone())
                    .filter(|x| x.is_ok())
                    .map(|x| x.unwrap())
                {
                    pcond = pcond.add(Expr::col(col.clone()).eq(value));
                }
                if !pcond.is_empty() {
                    cond = cond.add(pcond);
                }
            }
            _ => {}
        }
    }

    Ok(cond)
}
