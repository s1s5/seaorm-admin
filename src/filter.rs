use crate::Result;
use std::collections::HashMap;
// use itertools::Itertools;
use sea_orm::sea_query::{Condition, Expr};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, Iden, QueryFilter, QueryOrder, Select};

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

pub fn filter_by_hash_map<E>(
    qs: Select<E>,
    columns: &Vec<<E as EntityTrait>::Column>,
    filter: &HashMap<String, Vec<String>>,
) -> Result<Select<E>>
where
    E: EntityTrait,
{
    let mut cond = Condition::all();

    for col in columns.iter() {
        // TODO: support "like", "gt", etc..
        if let Some(queries) = filter.get(&col.to_string()) {
            let mut pcond = Condition::any();
            for value in queries {
                pcond = pcond.add(Expr::col(*col).eq(value.to_string()));
            }
            cond = cond.add(pcond);
        }
    }

    Ok(if !cond.is_empty() {
        qs.filter(cond)
    } else {
        qs
    })
}

pub fn filter_by_columns<M>(
    qs: Select<<M as ActiveModelTrait>::Entity>,
    columns: &Vec<<<M as ActiveModelTrait>::Entity as EntityTrait>::Column>,
    filter: &M,
    check_exists: bool,
) -> Result<Select<<M as ActiveModelTrait>::Entity>>
where
    M: ActiveModelTrait,
{
    let mut qs = qs;
    for col in columns.iter() {
        if let sea_orm::ActiveValue::Set(value) = filter.get(*col) {
            qs = qs.filter(col.eq(value));
        } else if check_exists {
            return Err(anyhow::anyhow!("key not found"));
        }
    }

    Ok(qs)
}

pub fn search_by_queries<E>(
    qs: Select<E>,
    columns: &Vec<<E as EntityTrait>::Column>,
    queries: &Vec<String>,
) -> Result<Select<E>>
where
    E: EntityTrait,
{
    let mut cond = Condition::any();
    for col in columns {
        match col.def().get_column_type() {
            sea_orm::ColumnType::Char(_)
            | sea_orm::ColumnType::String(_)
            | sea_orm::ColumnType::Text
            | sea_orm::ColumnType::Uuid => {
                let mut pcond = Condition::all();
                for value in queries {
                    pcond = pcond.add(Expr::col(*col).like(format!("%{}%", value.to_string())));
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
                    pcond = pcond.add(Expr::col(*col).eq(value));
                }
                if !pcond.is_empty() {
                    cond = cond.add(pcond);
                }
            }
            _ => {}
        }
    }

    Ok(if !cond.is_empty() {
        qs.filter(cond)
    } else {
        qs
    })
}
