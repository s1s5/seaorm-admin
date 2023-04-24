use std::collections::HashMap;

use super::{ListQuery, Result};

pub fn parse_query(m: &HashMap<String, Vec<String>>, list_per_page: u64) -> Result<ListQuery> {
    let filter: HashMap<_, _> = m
        .iter()
        .filter(|(key, _)| !key.starts_with('_'))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let queries: Vec<_> = m
        .get("_q")
        .map(|x| {
            x.iter()
                .flat_map(|s| s.trim().split(' '))
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or(vec![])
        .into_iter()
        .filter(|x| x.len() > 0)
        .collect();

    let page = m
        .get("_p")
        .filter(|x| x.len() == 1)
        .map(|x| x[0].clone())
        .and_then(|x| x.parse::<u64>().ok())
        .unwrap_or(0);

    Ok(ListQuery {
        filter,
        queries,
        offset: page * list_per_page,
        limit: list_per_page,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        let query = parse_query(
            &HashMap::from_iter([
                ("name".to_string(), vec!["hoge".to_string()]),
                (
                    "_q".to_string(),
                    vec!["search_word0".to_string(), "search_word1".to_string()],
                ),
                ("_p".to_string(), vec!["2".to_string()]),
            ]),
            20,
        )
        .expect("parse failed");
        assert_eq!(
            query.filter,
            HashMap::from_iter([("name".to_string(), vec!["hoge".to_string()]),])
        );
        assert_eq!(
            query.queries,
            vec!["search_word0".to_string(), "search_word1".to_string()]
        );
        assert_eq!(query.offset, 40);
        assert_eq!(query.limit, 20);
    }
}
