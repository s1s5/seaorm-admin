use super::{
    checkbox_field::CheckboxField, date_time_field::DateTimeField, input_field::InputField,
    textarea_field::TextareaField, timestamp_field::TimestampField, AdminField,
};
use crate::Result;
use log::warn;
use sea_orm::ColumnType;

pub fn get_default_field(name: &str, column_type: &ColumnType) -> Result<AdminField> {
    Ok(AdminField::Field(match column_type {
        sea_orm::ColumnType::Char(o) | sea_orm::ColumnType::String(o) => {
            if let Some(max_length) = o {
                Box::new(InputField::new_for_char(name, *max_length))
            } else {
                Box::new(TextareaField::new(name))
            }
        }
        sea_orm::ColumnType::Text | sea_orm::ColumnType::Json | sea_orm::ColumnType::JsonBinary => {
            Box::new(TextareaField::new(name))
        }
        sea_orm::ColumnType::TinyInteger
        | sea_orm::ColumnType::SmallInteger
        | sea_orm::ColumnType::Integer
        | sea_orm::ColumnType::BigInteger
        | sea_orm::ColumnType::TinyUnsigned
        | sea_orm::ColumnType::SmallUnsigned
        | sea_orm::ColumnType::Unsigned
        | sea_orm::ColumnType::BigUnsigned => Box::new(InputField::new_for_int(name)),
        sea_orm::ColumnType::Float
        | sea_orm::ColumnType::Double
        | sea_orm::ColumnType::Decimal(_)
        | sea_orm::ColumnType::Money(_) => Box::new(InputField::new_for_float(name)),
        sea_orm::ColumnType::DateTime => Box::new(DateTimeField::new(name)),
        sea_orm::ColumnType::TimestampWithTimeZone => Box::new(TimestampField::new(name)),

        sea_orm::ColumnType::Time => Box::new(InputField::new_with_type(name, "time")),
        sea_orm::ColumnType::Date => Box::new(InputField::new_with_type(name, "date")),
        sea_orm::ColumnType::Year(_o) => Box::new(InputField::new_with_type(name, "number")),
        sea_orm::ColumnType::Binary(_) | sea_orm::ColumnType::VarBinary(_) => {
            // pattern="^[A-Za-z0-9+/]{4}*[A-Za-z0-9+/]{4}([A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$"
            Box::new(TextareaField::new(name))
        }
        sea_orm::ColumnType::Boolean => Box::new(CheckboxField::new(name)),
        sea_orm::ColumnType::Uuid => Box::new(InputField::new_for_uuid(name)),
        _ => {
            warn!("Unsuported column type: {:?}", column_type);
            return Err(anyhow::anyhow!("Unsupported column type"));
        }
    }))
}
