use crate::{CustomError, Error, Json, Result};
use base64::Engine;
use log::warn;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DeriveIden, EntityTrait, ModelTrait};

// ----------------------------------------------------------------------------
pub fn set_from_json<M>(
    target: &mut M,
    columns: &Vec<<<M as ActiveModelTrait>::Entity as EntityTrait>::Column>,
    src: &Json,
) -> Result<()>
where
    M: ActiveModelTrait,
{
    columns
        .iter()
        .map(|col| {
            if let Some(v) = src.get(col.to_string()) {
                println!("col:{:?}, value:{:?}", col, sanitize_value(col, v));
                target.set(*col, sanitize_value(col, v)?);
            }
            Ok(())
        })
        .collect::<Result<()>>()
}

// ----------------------------------------------------------------------------
pub fn to_json<M>(
    model: &M,
    columns: &Vec<<<M as ModelTrait>::Entity as EntityTrait>::Column>,
) -> Result<Json>
where
    M: ModelTrait,
{
    let mut m = serde_json::Map::new();
    for c in columns.iter() {
        m.insert(c.to_string(), to_json_value(model.get(*c))?);
    }
    Ok(Json::Object(m))
}

// ----------------------------------------------------------------------------
pub fn json_force_str(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "".into(),
        serde_json::Value::Bool(b) => {
            if *b {
                "true".into()
            } else {
                "false".into()
            }
        }
        serde_json::Value::Number(n) => format!("{}", n),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(_a) => "[...]".into(),
        serde_json::Value::Object(_o) => "{...}".into(),
    }
}

// ----------------------------------------------------------------------------
pub fn json_overwrite_key(
    value: &serde_json::Value,
    overwrite: &serde_json::Value,
) -> Result<Json> {
    let mut o = value
        .as_object()
        .ok_or(CustomError::new("value not object"))?
        .clone();
    overwrite
        .as_object()
        .ok_or(CustomError::new("value not object"))?
        .iter()
        .for_each(|(key, value)| {
            if o.contains_key(key) {
                o.remove(key);
            }
            o.insert(key.clone(), value.clone());
        });
    Ok(Json::Object(o))
}

// ----------------------------------------------------------------------------
pub fn json_convert_vec_to_json(
    model: &Box<dyn super::ModelAdminTrait + Send + Sync>,
    total: u64,
    object_list: Vec<Json>,
) -> Result<Json> {
    let object_list = object_list
        .into_iter()
        .map(|x| {
            Ok(serde_json::Value::Object(serde_json::Map::from_iter(
                [
                    (
                        "key".to_string(),
                        serde_json::Value::String(model.json_to_key(&x)?),
                    ),
                    (
                        "label".to_string(),
                        serde_json::Value::String(model.to_str(&x)?),
                    ),
                    ("data".to_string(), x),
                ]
                .into_iter(),
            )))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(serde_json::json!({
        "total" : total,
        "data": Json::Array(object_list),
    }))
}

// ============================================================================
macro_rules! sanitize_value_check_empty {
    ($ident: ident, $col: expr, $v: expr) => {
        if json_value_is_empty($v) {
            if $col.def().is_null() {
                return Ok(sea_orm::Value::$ident(None));
            } else {
                return Err(Box::new(sea_orm::DbErr::Custom(format!(
                    "{:?} cannot be null",
                    $col.to_string()
                ))));
            }
        }
    };
}

fn sanitize_value<C>(col: &C, v: &Json) -> Result<sea_orm::Value>
where
    C: sea_orm::ColumnTrait,
{
    Ok(match col.def().get_column_type() {
        sea_orm::ColumnType::Char(_)
        | sea_orm::ColumnType::String(_)
        | sea_orm::ColumnType::Text => {
            if json_value_is_empty(v) {
                match col.def().get_column_type() {
                    sea_orm::ColumnType::Char(_)
                    | sea_orm::ColumnType::String(_)
                    | sea_orm::ColumnType::Text => {
                        if !col.def().is_null() {
                            return Ok(sea_orm::Value::String(Some(Box::new("".to_string()))));
                        }
                    }
                    _ => {}
                }
                return Ok(sea_orm::Value::String(None));
            }
            let v: String = serde_json::from_value(v.clone())?;
            sea_orm::Value::String(Some(Box::new(v)))
        }
        sea_orm::ColumnType::TinyInteger => {
            sanitize_value_check_empty!(TinyInt, col, v);
            sea_orm::Value::TinyInt(Some(parse_value::<i8>(v)?))
        }
        sea_orm::ColumnType::SmallInteger => {
            sanitize_value_check_empty!(SmallInt, col, v);
            sea_orm::Value::SmallInt(Some(parse_value::<i16>(v)?))
        }
        sea_orm::ColumnType::Integer => {
            sanitize_value_check_empty!(Int, col, v);
            sea_orm::Value::Int(Some(parse_value::<i32>(v)?))
        }
        sea_orm::ColumnType::BigInteger => {
            sanitize_value_check_empty!(BigInt, col, v);
            sea_orm::Value::BigInt(Some(parse_value::<i64>(v)?))
        }
        sea_orm::ColumnType::TinyUnsigned => {
            sanitize_value_check_empty!(TinyUnsigned, col, v);
            sea_orm::Value::TinyUnsigned(Some(parse_value::<u8>(v)?))
        }
        sea_orm::ColumnType::SmallUnsigned => {
            sanitize_value_check_empty!(SmallUnsigned, col, v);
            sea_orm::Value::SmallUnsigned(Some(parse_value::<u16>(v)?))
        }
        sea_orm::ColumnType::Unsigned => {
            sanitize_value_check_empty!(Unsigned, col, v);
            sea_orm::Value::Unsigned(Some(parse_value::<u32>(v)?))
        }
        sea_orm::ColumnType::BigUnsigned => {
            sanitize_value_check_empty!(BigUnsigned, col, v);
            sea_orm::Value::BigUnsigned(Some(parse_value::<u64>(v)?))
        }
        sea_orm::ColumnType::Float => {
            sanitize_value_check_empty!(Float, col, v);
            sea_orm::Value::Float(Some(parse_value::<f32>(v)?))
        }
        sea_orm::ColumnType::Double => {
            sanitize_value_check_empty!(Double, col, v);
            sea_orm::Value::Double(Some(parse_value::<f64>(v)?))
        }
        sea_orm::ColumnType::Decimal(_o) => {
            sanitize_value_check_empty!(Decimal, col, v);
            let v: Decimal = serde_json::from_value(v.clone())?;
            sea_orm::Value::Decimal(Some(Box::new(v)))
        }
        sea_orm::ColumnType::DateTime | sea_orm::ColumnType::Timestamp => {
            sanitize_value_check_empty!(ChronoDateTime, col, v);
            let v: chrono::NaiveDateTime = serde_json::from_value(v.clone())?;
            sea_orm::Value::ChronoDateTime(Some(Box::new(v)))
        }
        sea_orm::ColumnType::TimestampWithTimeZone => {
            sanitize_value_check_empty!(ChronoDateTimeWithTimeZone, col, v);
            let v: chrono::DateTime<chrono::FixedOffset> = serde_json::from_value(v.clone())?;
            sea_orm::Value::ChronoDateTimeWithTimeZone(Some(Box::new(v)))
        }
        sea_orm::ColumnType::Time => {
            sanitize_value_check_empty!(ChronoTime, col, v);
            let v: chrono::NaiveTime = serde_json::from_value(v.clone())?;
            sea_orm::Value::ChronoTime(Some(Box::new(v)))
        }
        sea_orm::ColumnType::Date => {
            sanitize_value_check_empty!(ChronoDate, col, v);
            let v: chrono::NaiveDate = serde_json::from_value(v.clone())?;
            sea_orm::Value::ChronoDate(Some(Box::new(v)))
        }
        // sea_orm::ColumnType::Year(o) => {}
        // sea_orm::ColumnType::Interval(o) => {}
        sea_orm::ColumnType::Binary(_o) => {
            sanitize_value_check_empty!(Bytes, col, v);
            let v: String = serde_json::from_value(v.clone())?;
            let decoded = base64::engine::general_purpose::STANDARD.decode(&v)?;
            sea_orm::Value::Bytes(Some(Box::new(decoded)))
        }
        // sea_orm::ColumnType::VarBinary(o) => {}
        // sea_orm::ColumnType::Bit(o) => {}
        // sea_orm::ColumnType::VarBit(o) => {}
        sea_orm::ColumnType::Boolean => {
            sanitize_value_check_empty!(Bool, col, v);
            let v: bool = serde_json::from_value(v.clone())?;
            sea_orm::Value::Bool(Some(v))
        }
        // sea_orm::ColumnType::Money(o) => {}
        sea_orm::ColumnType::Json | sea_orm::ColumnType::JsonBinary => {
            sanitize_value_check_empty!(Json, col, v);
            let v: String = serde_json::from_value(v.clone())?;
            let v: Json = serde_json::from_str(&v)?;
            sea_orm::Value::Json(Some(Box::new(v.clone())))
        }
        sea_orm::ColumnType::Uuid => {
            sanitize_value_check_empty!(Uuid, col, v);
            let v: uuid::Uuid = serde_json::from_value(v.clone())?;
            sea_orm::Value::Uuid(Some(Box::new(v)))
        }
        // sea_orm::ColumnType::Array(o) => {}
        // sea_orm::ColumnType::Cidr => {}
        // sea_orm::ColumnType::Inet => {}
        // sea_orm::ColumnType::MacAddr => {}
        _ => {
            warn!(
                "Unsupported column type found. col={:?}, v={:?}",
                col.def(),
                v
            );
            return Err(Box::new(sea_orm::DbErr::Custom("not implemented".into())));
        }
    })
}

fn parse_value<'a, T>(value: &'a Json) -> Result<T>
where
    T: std::str::FromStr + serde::de::DeserializeOwned,
    <T as std::str::FromStr>::Err: std::error::Error + 'static,
{
    if value.is_string() {
        value
            .as_str()
            .unwrap()
            .parse::<T>()
            .map_err(|x| Box::new(x) as Error)
    } else {
        serde_json::from_value::<T>(value.clone()).map_err(|x| Box::new(x) as Error)
    }
}

fn json_value_is_empty(value: &serde_json::Value) -> bool {
    value.is_null() || (value.is_string() && value.as_str().unwrap().len() == 0)
}

fn to_json_value(value: sea_orm::Value) -> Result<Json> {
    match value {
        sea_orm::Value::Bool(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::TinyInt(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::SmallInt(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::Int(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::BigInt(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::TinyUnsigned(v) => {
            serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
        }
        sea_orm::Value::SmallUnsigned(v) => {
            serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
        }
        sea_orm::Value::Unsigned(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::BigUnsigned(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::Float(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::Double(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::String(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::Char(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::Bytes(v) => {
            if let Some(v) = v {
                let encoded = base64::engine::general_purpose::STANDARD.encode(&v[..]);
                serde_json::to_value(encoded).map_err(|e| Box::new(e) as Error)
            } else {
                serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
            }
        }
        sea_orm::Value::Json(v) => {
            if let Some(v) = v {
                let json_str =
                    serde_json::to_string_pretty(&v).map_err(|e| Box::new(e) as Error)?;
                serde_json::to_value(json_str).map_err(|e| Box::new(e) as Error)
            } else {
                serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
            }
        }
        sea_orm::Value::ChronoDate(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::ChronoTime(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::ChronoDateTime(v) => {
            serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
        }
        sea_orm::Value::ChronoDateTimeUtc(v) => {
            serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
        }
        sea_orm::Value::ChronoDateTimeLocal(v) => {
            serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
        }
        sea_orm::Value::ChronoDateTimeWithTimeZone(v) => {
            serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
        }
        sea_orm::Value::TimeDate(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::TimeTime(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::TimeDateTime(v) => {
            serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
        }
        sea_orm::Value::TimeDateTimeWithTimeZone(v) => {
            serde_json::to_value(v).map_err(|e| Box::new(e) as Error)
        }
        sea_orm::Value::Uuid(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        sea_orm::Value::Decimal(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        // sea_orm::Value::BigDecimal(v) => serde_json::to_value(v).map_err(|e| Box::new(e) as Error),
        // sea_orm::Value::Array(ty, v) => {}
        // sea_orm::Value::IpNetwork(v) => {}
        // sea_orm::Value::MacAddress(v) => {}
        _ => {
            warn!("Unsupported column type found. {:?}", value);
            Err(Box::new(sea_orm::DbErr::Custom("Unsupported".into())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Iterable, Set, TryIntoModel};
    use serde_json::json;

    mod test_model {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(rs_type = "String", db_type = "String(None)")]
        pub enum Category {
            #[sea_orm(string_value = "B")]
            Big,
            #[sea_orm(string_value = "S")]
            Small,
        }

        #[derive(Clone, Debug, PartialEq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(rs_type = "i32", db_type = "Integer")]
        pub enum Color {
            #[sea_orm(num_value = 0)]
            Black,
            #[sea_orm(num_value = 1)]
            White,
        }

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_model")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,

            // field test
            #[sea_orm(column_type = "Char(Some(32))", nullable)]
            pub char_f: Option<String>,

            pub string_f: Option<String>,
            pub text_f: Option<String>,
            pub tiny_integer_f: Option<i16>,
            pub small_integer_f: Option<i16>,
            pub integer_f: Option<i32>,
            pub big_integer_f: Option<i64>,
            pub tiny_unsigned_f: Option<i16>,
            pub small_unsigned_f: Option<i16>,
            pub unsigned_f: Option<i32>,
            pub big_unsigned_f: Option<i64>,

            #[sea_orm(column_type = "Float", nullable)]
            pub float_f: Option<f32>,

            #[sea_orm(column_type = "Double", nullable)]
            pub double_f: Option<f64>,

            #[sea_orm(column_type = "Decimal(Some((32, 2)))", nullable)]
            pub decimal_f: Option<Decimal>,
            pub date_time_f: Option<DateTime>,
            pub timestamp_f: Option<DateTime>,
            pub timestamp_with_time_zone_f: Option<DateTimeWithTimeZone>,
            pub time_f: Option<Time>,
            pub date_f: Option<Date>,
            #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
            pub binary_f: Option<Vec<u8>>,

            pub boolean_f: Option<bool>,
            pub json_f: Option<Json>,
            #[sea_orm(column_type = "JsonBinary", nullable)]
            pub json_binary_f: Option<Json>,
            pub uuid_f: Option<Uuid>,

            // enum test
            pub enum_string: Option<Category>,
            pub enum_i32: Option<Color>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    mod nulltest {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "nulltest_model")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,

            #[sea_orm(column_type = "Char(Some(32))", nullable)]
            pub nullable_char_f: Option<String>,

            #[sea_orm(column_type = "Char(Some(32))")]
            pub nonnull_char_f: String,

            pub nullable_integer_f: Option<i32>,
            pub nonnull_integer_f: i32,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    mod test_model_admin {
        #[derive(crate::ModelAdmin, Default)]
        #[model_admin(module = super::test_model)]
        pub struct TestModelAdmin;

        #[derive(crate::ModelAdmin, Default)]
        #[model_admin(module = super::nulltest)]
        pub struct NullTestAdmin;
    }

    #[test]
    fn test_serialize() {
        let fields = test_model::Column::iter().collect();
        let now = chrono::DateTime::parse_from_rfc3339("2022-01-01T12:00:00.123456Z")
            .expect("Failed to parse datetime string")
            .with_timezone(&chrono::Utc);

        let mut a = test_model::ActiveModel {
            ..Default::default()
        };

        a.id = Set(1);
        a.char_f = Set(Some("char field".to_string()));
        a.string_f = Set(Some("string field".to_string()));
        a.text_f = Set(Some("text field".to_string()));
        a.tiny_integer_f = Set(Some(1));
        a.small_integer_f = Set(Some(2));
        a.integer_f = Set(Some(3));
        a.big_integer_f = Set(Some(4));
        a.tiny_unsigned_f = Set(Some(5));
        a.small_unsigned_f = Set(Some(6));
        a.unsigned_f = Set(Some(7));
        a.big_unsigned_f = Set(Some(8));
        a.float_f = Set(Some(0.1));
        a.double_f = Set(Some(0.01));
        a.decimal_f = Set(Some(Decimal::new(33, 2)));
        a.date_time_f = Set(Some(now.naive_utc()));
        a.timestamp_f = Set(Some(now.naive_utc()));
        a.timestamp_with_time_zone_f = Set(Some(
            now.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
        ));
        a.time_f = Set(Some(chrono::NaiveTime::from_hms_opt(13, 30, 0).unwrap()));
        a.date_f = Set(Some(chrono::NaiveDate::from_ymd_opt(2023, 4, 1).unwrap()));
        a.binary_f = Set(Some(vec![1, 2, 3, 4]));
        a.boolean_f = Set(Some(true));
        a.json_f = Set(Some(serde_json::json!({"a": "b", "c": 3})));
        a.json_binary_f = Set(Some(serde_json::json!({"d": "e", "f": 8})));
        a.uuid_f = Set(Some(uuid::Uuid::from_bytes([
            0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6,
            0xd7, 0xd8,
        ])));
        a.enum_string = Set(Some(test_model::Category::Big));
        a.enum_i32 = Set(Some(test_model::Color::Black));

        let a: test_model::Model = a
            .try_into_model()
            .expect("failed to convert ActiveModel to Model.");

        let jv: Json = to_json(&a, &fields).expect("failed to serialize to json");

        assert!(jv.is_object());
        let o = jv.as_object().unwrap();

        for (key, value) in [
            ("big_integer_f", json!(4)),
            ("big_unsigned_f", json!(8)),
            ("binary_f", json!("AQIDBA==")),
            ("boolean_f", json!(true)),
            ("char_f", json!("char field")),
            ("date_f", json!("2023-04-01")),
            ("date_time_f", json!("2022-01-01T12:00:00.123456")),
            ("decimal_f", json!("0.33")),
            ("double_f", json!(0.01)),
            ("enum_i32", json!(0)),
            ("enum_string", json!("B")),
            ("float_f", json!(0.10000000149011612)),
            ("id", json!(1)),
            ("integer_f", json!(3)),
            ("json_binary_f", json!("{\n  \"d\": \"e\",\n  \"f\": 8\n}")),
            ("json_f", json!("{\n  \"a\": \"b\",\n  \"c\": 3\n}")),
            ("small_integer_f", json!(2)),
            ("small_unsigned_f", json!(6)),
            ("string_f", json!("string field")),
            ("text_f", json!("text field")),
            ("time_f", json!("13:30:00")),
            ("timestamp_f", json!("2022-01-01T12:00:00.123456")),
            (
                "timestamp_with_time_zone_f",
                json!("2022-01-01T12:00:00.123456+00:00"),
            ),
            ("tiny_integer_f", json!(1)),
            ("tiny_unsigned_f", json!(5)),
            ("unsigned_f", json!(7)),
            ("uuid_f", json!("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8")),
        ] {
            assert_eq!(o.get(key).expect("key not found"), &value);
        }

        let mut b = test_model::ActiveModel {
            ..Default::default()
        };
        set_from_json(&mut b, &fields, &jv).expect("set_from_json failed");
        let b: test_model::Model = b
            .try_into_model()
            .expect("failed to convert ActiveModel to Model");

        assert_eq!(b.id, a.id);
        assert_eq!(b.char_f, a.char_f);
        assert_eq!(b.string_f, a.string_f);
        assert_eq!(b.text_f, a.text_f);
        assert_eq!(b.tiny_integer_f, a.tiny_integer_f);
        assert_eq!(b.small_integer_f, a.small_integer_f);
        assert_eq!(b.integer_f, a.integer_f);
        assert_eq!(b.big_integer_f, a.big_integer_f);
        assert_eq!(b.tiny_unsigned_f, a.tiny_unsigned_f);
        assert_eq!(b.small_unsigned_f, a.small_unsigned_f);
        assert_eq!(b.unsigned_f, a.unsigned_f);
        assert_eq!(b.big_unsigned_f, a.big_unsigned_f);
        assert_eq!(b.float_f, a.float_f);
        assert_eq!(b.double_f, a.double_f);
        assert_eq!(b.decimal_f, a.decimal_f);
        assert_eq!(b.date_time_f, a.date_time_f);
        assert_eq!(b.timestamp_f, a.timestamp_f);
        assert_eq!(b.timestamp_with_time_zone_f, a.timestamp_with_time_zone_f);
        assert_eq!(b.time_f, a.time_f);
        assert_eq!(b.date_f, a.date_f);
        assert_eq!(b.binary_f, a.binary_f);
        assert_eq!(b.boolean_f, a.boolean_f);
        assert_eq!(b.json_f, a.json_f);
        assert_eq!(b.json_binary_f, a.json_binary_f);
        assert_eq!(b.uuid_f, a.uuid_f);
        assert_eq!(b.enum_string, a.enum_string);
        assert_eq!(b.enum_i32, a.enum_i32);
    }

    #[test]
    fn test_json_overwrite_key() {
        let value = json!({
            "a": 1,
            "b": 2,
        });
        let overwrite = json!({
            "a": 3,
        });

        let updated = json_overwrite_key(&value, &overwrite).expect("failed to overwrite");

        assert_eq!(updated["a"], 3);
        assert_eq!(updated["b"], 2);
    }

    #[test]
    fn test_nulltest() {
        let fields = nulltest::Column::iter().collect();
        let mut a = nulltest::ActiveModel {
            ..Default::default()
        };
        let mut jv = json!({
            "id": "1",
            "nullable_char_f": "",
            "nonnull_char_f": "",
            "nullable_integer_f": "",
            "nonnull_integer_f": "1",
        });

        set_from_json(&mut a, &fields, &jv).expect("set_from_json failed");
        let b: nulltest::Model = a
            .clone()
            .try_into_model()
            .expect("failed to convert ActiveModel to Model");
        assert_eq!(b.id, 1);
        assert_eq!(b.nullable_char_f, None);
        assert_eq!(b.nonnull_char_f, "".to_string());
        assert_eq!(b.nullable_integer_f, None);
        assert_eq!(b.nonnull_integer_f, 1);

        jv["nonnull_integer_f"] = Json::String("".to_string());

        let set_result = set_from_json(&mut a, &fields, &jv);
        assert_eq!(set_result.is_err(), true);
    }

    // #[test]
    // fn test_json_convert_vec_to_json() {
    // なんでだめ？？
    //     let model_admin: Box<dyn crate::ModelAdminTrait + Send + Sync> =
    //         Box::new(test_model_admin::TestModelAdmin);
    //     let v = json_convert_vec_to_json(
    //         &model_admin,
    //         1,
    //         vec![json!({
    //             "id": 1,
    //         })],
    //     );
    //     println!("{:?}", v);
    // }
}
