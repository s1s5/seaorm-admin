// use std::collections::HashMap;

// use askama::DynTemplate;
// use chrono::Timelike;
// use log::warn;

// use crate::{
//     json_force_str,
//     templates::{
//         AdminFormCheckbox, AdminFormDatetimeInput, AdminFormDatetimeInputValue, AdminFormInput,
//         AdminFormSelect, AdminFormTextarea,
//     },
//     AdminField, CustomError, Json, Result,
// };

// pub trait Widget {
//     fn create(&self, field: &AdminField, value: Option<&Json>) -> Result<Box<dyn DynTemplate>>;
// }

// pub struct DefaultWidget;

// impl Widget for DefaultWidget {
//     fn create(&self, field: &AdminField, value: Option<&Json>) -> Result<Box<dyn DynTemplate>> {
//         let org_value = value;
//         let value = value.map(|x| json_force_str(&x));
//         let mut input = Box::new(AdminFormInput {
//             name: field.name.clone(),
//             label: field.name.clone(),
//             r#type: "text".into(),
//             value: value.clone(),
//             help_text: None,
//             disabled: !field.editable,
//             attributes: HashMap::new(),
//         });
//         let textarea = Box::new(AdminFormTextarea {
//             name: field.name.clone(),
//             label: field.name.clone(),
//             value: value.clone(),
//             help_text: None,
//             disabled: !field.editable,
//         });

//         Ok(match &field.column_type {
//             sea_orm::ColumnType::Char(o) | sea_orm::ColumnType::String(o) => {
//                 if let Some(o) = o {
//                     input.attributes =
//                         HashMap::from_iter([("maxlength".to_string(), o.to_string())]);
//                     input
//                 } else {
//                     textarea
//                 }
//             }
//             sea_orm::ColumnType::Text
//             | sea_orm::ColumnType::Json
//             | sea_orm::ColumnType::JsonBinary => textarea,
//             sea_orm::ColumnType::TinyInteger
//             | sea_orm::ColumnType::SmallInteger
//             | sea_orm::ColumnType::Integer
//             | sea_orm::ColumnType::BigInteger
//             | sea_orm::ColumnType::TinyUnsigned
//             | sea_orm::ColumnType::SmallUnsigned
//             | sea_orm::ColumnType::Unsigned
//             | sea_orm::ColumnType::BigUnsigned => {
//                 input.r#type = "number".into();
//                 input.attributes = HashMap::from_iter([("step".into(), "1".into())]);
//                 input
//             }
//             sea_orm::ColumnType::Float
//             | sea_orm::ColumnType::Double
//             | sea_orm::ColumnType::Decimal(_)
//             | sea_orm::ColumnType::Money(_) => {
//                 input.r#type = "number".into();
//                 input.attributes = HashMap::from_iter([("step".into(), "auto".into())]);
//                 input
//             }
//             sea_orm::ColumnType::DateTime | sea_orm::ColumnType::TimestampWithTimeZone => {
//                 let value = if let Some(value) = &value {
//                     if field.column_type == sea_orm::ColumnType::DateTime {
//                         let v: Option<chrono::NaiveDateTime> =
//                             serde_json::from_value(Json::String(value.clone())).ok();
//                         if let Some(v) = v {
//                             Some(AdminFormDatetimeInputValue {
//                                 raw: value.clone(),
//                                 datetime_without_seconds: v.format("%Y-%m-%dT%H:%M").to_string(),
//                                 seconds: v.time().second() as f64
//                                     + (v.timestamp_subsec_micros() as f64) * 1.0e-6,
//                                 timezone: 0,
//                             })
//                         } else {
//                             None
//                         }
//                     } else {
//                         let v: Option<chrono::DateTime<chrono::FixedOffset>> =
//                             serde_json::from_value(Json::String(value.clone())).ok();
//                         if let Some(v) = v {
//                             Some(AdminFormDatetimeInputValue {
//                                 raw: value.clone(),
//                                 datetime_without_seconds: v.format("%Y-%m-%dT%H:%M").to_string(),
//                                 seconds: v.time().second() as f64
//                                     + (v.timestamp_subsec_micros() as f64) * 1.0e-6,
//                                 timezone: v.timezone().local_minus_utc() * 60,
//                             })
//                         } else {
//                             None
//                         }
//                     }
//                 } else {
//                     None
//                 };
//                 Box::new(AdminFormDatetimeInput {
//                     name: field.name.clone(),
//                     label: field.name.clone(),
//                     value: value,
//                     with_timezone: field.column_type == sea_orm::ColumnType::TimestampWithTimeZone,
//                     help_text: None,
//                     disabled: !field.editable,
//                 })
//             }

//             sea_orm::ColumnType::Time => {
//                 input.r#type = "time".into();
//                 input
//             }
//             sea_orm::ColumnType::Date => {
//                 input.r#type = "date".into();
//                 input
//             }
//             sea_orm::ColumnType::Year(_o) => {
//                 input.r#type = "number".into();
//                 input
//             }
//             // sea_orm::ColumnType::Interval(o) => {}
//             sea_orm::ColumnType::Binary(_) | sea_orm::ColumnType::VarBinary(_) => {
//                 // pattern="^[A-Za-z0-9+/]{4}*[A-Za-z0-9+/]{4}([A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$"
//                 textarea
//                 // input.attributes = HashMap::from_iter([
//                 //     ("pattern".into(), "[0-9a-fA-F]*".into()),
//                 //     ("title".into(), "input format must be hex string".into()),
//                 // ]);
//                 // input
//             }
//             // sea_orm::ColumnType::Bit(_o) => {
//             //     input.attributes = HashMap::from_iter([
//             //         ("pattern".into(), "[0-1]*".into()),
//             //         ("title".into(), "input format must be bit string".into()),
//             //     ]);
//             //     input
//             // }
//             // sea_orm::ColumnType::VarBit(_o) => {
//             //     input.attributes = HashMap::from_iter([
//             //         ("pattern".into(), "[0-1]*".into()),
//             //         ("title".into(), "input format must be bit string".into()),
//             //     ]);
//             //     input
//             // }
//             sea_orm::ColumnType::Boolean => Box::new(AdminFormCheckbox {
//                 name: field.name.clone(),
//                 label: field.name.clone(),
//                 checked: org_value
//                     .unwrap_or(&Json::Bool(false))
//                     .as_bool()
//                     .unwrap_or(false),
//                 help_text: None,
//                 disabled: !field.editable,
//             }),
//             //  => {}
//             sea_orm::ColumnType::Uuid => {
//                 input.attributes = HashMap::from_iter([
//                 (
//                     "pattern".into(),
//                     "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
//                         .into(),
//                 ),
//                 (
//                     "title".into(),
//                     "input format must be 'xxxxxxxx-xxxx-Mxxx-Nxxx-xxxxxxxxxxxx'".into(),
//                 ),
//             ]);
//                 input
//             }
//             // sea_orm::ColumnType::Custom(DynIden),
//             // sea_orm::ColumnType::Enum {
//             //     name: DynIden,
//             //     variants: Vec<DynIden>,
//             // },
//             // sea_orm::ColumnType::Array(o) => {}
//             // sea_orm::ColumnType::Cidr => {
//             //     input.attributes = HashMap::from_iter([
//             //         // TODO: ("pattern".into(), "[0-9a-fA-F]*".into()),
//             //         ("title".into(), "input format must be cidr".into()),
//             //     ]);
//             //     input
//             // }
//             // sea_orm::ColumnType::Inet => {
//             //     input.attributes = HashMap::from_iter([
//             //         // TODO: ("pattern".into(), "[0-9a-fA-F]*".into()),
//             //         ("title".into(), "input format must be inet".into()),
//             //     ]);
//             //     input
//             // }
//             // sea_orm::ColumnType::MacAddr => {
//             //     input.attributes = HashMap::from_iter([
//             //         // TODO: ("pattern".into(), "[0-9a-fA-F]*".into()),
//             //         ("title".into(), "input format must be macaddr".into()),
//             //     ]);
//             //     input
//             // }
//             _ => {
//                 warn!("Unsuported column type: {:?}", field.column_type);
//                 return Err(Box::new(CustomError::new("Unsupported column type")));
//             }
//         })
//     }
// }

// pub struct HiddenWidget;
// impl Widget for HiddenWidget {
//     fn create(&self, field: &AdminField, value: Option<&Json>) -> Result<Box<dyn DynTemplate>> {
//         let value = value.map(|x| json_force_str(&x));
//         Ok(Box::new(AdminFormInput {
//             name: field.name.clone(),
//             label: field.name.clone(),
//             r#type: "hidden".into(),
//             value: value.clone(),
//             help_text: None,
//             disabled: !field.editable,
//             attributes: HashMap::new(),
//         }))
//     }
// }

// pub struct EnumWidget {
//     choices: Vec<(String, String)>,
// }

// impl EnumWidget {
//     pub fn new(choices: Vec<(String, String)>) -> Self {
//         EnumWidget { choices: choices }
//     }

//     pub fn from_enum<T>(it: T) -> Self
//     where
//         T: Iterator,
//         <T as Iterator>::Item: std::fmt::Debug + std::fmt::Display,
//     {
//         Self::new(
//             it.map(|x| {
//                 (
//                     x.to_string().trim_matches('\'').to_string(),
//                     format!("{:?}", x),
//                 )
//             })
//             .collect(),
//         )
//     }
// }

// impl Widget for EnumWidget {
//     fn create(&self, field: &AdminField, value: Option<&Json>) -> Result<Box<dyn DynTemplate>> {
//         let value = value.map(|x| json_force_str(&x)).unwrap_or("".to_string());
//         Ok(Box::new(AdminFormSelect {
//             name: field.name.clone(),
//             label: field.name.clone(),
//             value: value.clone(),
//             help_text: None,
//             disabled: !field.editable,
//             choices: self.choices.clone(),
//             attributes: HashMap::new(),
//         }))
//     }
// }
