use crate::Json;

pub fn get_value<'a>(parent_value: Option<&'a Json>, name: &str) -> Option<&'a Json> {
    parent_value
        .map(|x| x.as_object())
        .unwrap_or(None)
        .map(|y| y.get(name))
        .unwrap_or(None)
}
