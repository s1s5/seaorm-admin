use crate::{CustomError, Json, Result};

// ----------------------------------------------------------------------------
pub fn to_key_string<C>(columns: &Vec<C>, value: &Json) -> Result<String>
where
    C: sea_orm::ColumnTrait,
{
    Ok(columns
        .iter()
        .map(|c| {
            let v = value.get(c.to_string());
            if v.is_none() {
                return Err(anyhow::anyhow!("key not set"));
            }
            b62encode(c, v.unwrap())
        })
        .collect::<Result<Vec<_>>>()?
        .join("-"))
}

// ----------------------------------------------------------------------------
pub fn from_key_string<C>(columns: &Vec<C>, key: &str) -> Result<Json>
where
    C: sea_orm::ColumnTrait,
{
    let mut m = serde_json::Map::new();
    for (v, col) in key.split("-").zip(columns).into_iter() {
        m.insert(col.to_string(), b62decode(col, v)?);
    }
    Ok(Json::Object(m))
}

// ============================================================================
// macro_rules! b62encode_tostring {
//     ($i:expr) => {
//         if let Some(i) = $i {
//             i.to_string()
//         } else {
//             return Err(Box::new(CustomError::new("b62encode")));
//         }
//     };
// }

fn b62encode<C>(col: &C, value: &Json) -> Result<String>
where
    C: sea_orm::ColumnTrait,
{
    match col.def().get_column_type() {
        sea_orm::ColumnType::Char(_)
        | sea_orm::ColumnType::String(_)
        | sea_orm::ColumnType::Text => value
            .as_str()
            .ok_or(anyhow::anyhow!("parse json error"))
            .map(|x| base_62::encode(x.as_bytes())),
        sea_orm::ColumnType::TinyInteger
        | sea_orm::ColumnType::SmallInteger
        | sea_orm::ColumnType::Integer
        | sea_orm::ColumnType::BigInteger
        | sea_orm::ColumnType::TinyUnsigned
        | sea_orm::ColumnType::SmallUnsigned
        | sea_orm::ColumnType::Unsigned
        | sea_orm::ColumnType::BigUnsigned => value
            .as_i64()
            .ok_or(anyhow::anyhow!("parse json error"))
            .map(|x| x.to_string()),

        // sea_orm::ColumnType::Float => {}
        // sea_orm::ColumnType::Double => {}
        // sea_orm::ColumnType::Decimal(o) => {}
        // sea_orm::ColumnType::DateTime => {}
        // sea_orm::ColumnType::Timestamp => {}
        // sea_orm::ColumnType::TimestampWithTimeZone => {}
        // sea_orm::ColumnType::Time => {}
        // sea_orm::ColumnType::Date => {}
        // sea_orm::ColumnType::Year(o) => {}
        // sea_orm::ColumnType::Interval(i, o) => {},
        // sea_orm::ColumnType::Binary(o) => {}
        // sea_orm::ColumnType::VarBinary(o) => {}
        // sea_orm::ColumnType::Bit(o) => {}
        // sea_orm::ColumnType::VarBit(o) => {}
        // sea_orm::ColumnType::Boolean => {}
        // sea_orm::ColumnType::Money(o) => {}
        // sea_orm::ColumnType::Json => {}
        // sea_orm::ColumnType::JsonBinary => {}
        sea_orm::ColumnType::Uuid => {
            let uuid: uuid::Uuid = serde_json::from_value(value.clone())?;
            Ok(base_62::encode(uuid.as_bytes()))
        }
        // sea_orm::ColumnType::Custom(DynIden),
        // sea_orm::ColumnType::Enum
        // sea_orm::ColumnType::Array(SeaRc<ColumnType>),
        // sea_orm::ColumnType::Cidr,
        // sea_orm::ColumnType::Inet,
        // sea_orm::ColumnType::MacAddr,
        _ => Err(anyhow::anyhow!("Unsupported Column type for key_to_str",)),
    }
}

macro_rules! b62decode_parse {
    ($i:expr, $t: ty, $V: ident) => {
        serde_json::to_value($i.parse::<$t>().map_err(|x| Box::new(x))?).map_err(|x| x.into())
    };
}

fn b62decode<T>(col: &T, v: &str) -> Result<Json>
where
    T: sea_orm::ColumnTrait,
{
    match col.def().get_column_type() {
        sea_orm::ColumnType::TinyInteger => {
            b62decode_parse!(v, i8, TinyInt)
        }
        sea_orm::ColumnType::SmallInteger => {
            b62decode_parse!(v, i16, SmallInt)
        }
        sea_orm::ColumnType::Integer => {
            b62decode_parse!(v, i32, Int)
        }
        sea_orm::ColumnType::BigInteger => {
            b62decode_parse!(v, i64, BigInt)
        }
        sea_orm::ColumnType::TinyUnsigned => {
            b62decode_parse!(v, u8, TinyUnsigned)
        }
        sea_orm::ColumnType::SmallUnsigned => {
            b62decode_parse!(v, u16, SmallUnsigned)
        }
        sea_orm::ColumnType::Unsigned => {
            b62decode_parse!(v, u32, Unsigned)
        }
        sea_orm::ColumnType::BigUnsigned => {
            b62decode_parse!(v, u64, BigUnsigned)
        }
        sea_orm::ColumnType::Text
        | sea_orm::ColumnType::String(_)
        | sea_orm::ColumnType::Char(_) => Ok(Json::String(String::from_utf8(
            base_62::decode(v).map_err(|e| {
                Box::new(CustomError::new(format!("base_62::decode error: {:?}", e)))
            })?,
        )?)),
        sea_orm::ColumnType::Uuid => {
            let bytes = base_62::decode(v).map_err(|e| {
                Box::new(CustomError::new(format!("base_62::decode error: {:?}", e)))
            })?;
            Ok(serde_json::to_value(uuid::Uuid::from_slice(&bytes)?)?)
        }
        _ => Err(anyhow::anyhow!("b62decode")),
    }
}
