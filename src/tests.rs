// use crate::Admin;
// use admin_macro::ModelAdmin;

// mod bakery {
//     use sea_orm::entity::prelude::*;

//     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//     #[sea_orm(table_name = "bakery")]
//     pub struct Model {
//         #[sea_orm(primary_key)]
//         pub id: i32,

//         pub name: String,
//     }

//     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//     pub enum Relation {}

//     impl ActiveModelBehavior for ActiveModel {}
// }

// mod cake {
//     use sea_orm::entity::prelude::*;

//     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//     #[sea_orm(table_name = "cake")]
//     pub struct Model {
//         #[sea_orm(primary_key)]
//         pub id: i32,

//         pub name: String,
//         pub price: u32,
//     }

//     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//     pub enum Relation {}

//     impl ActiveModelBehavior for ActiveModel {}
// }

// // mod seaorm_admin {
// //     pub use crate::*;
// // }

// #[derive(ModelAdmin, Default)]
// #[model_admin(module = cake)]
// struct CakeAdmin;

// #[test]
// fn test_default() {
//     let mut admin = Admin::new(connection, "/admin");
//     admin.add_model(CakeAdmin);
// }

// #[test]
// fn test_list_display() {
//     let mut admin = Admin::new(connection, "/admin");
//     admin.add_model(CakeAdmin);
// }

// #[test]
// fn test_fields() {}

// #[test]
// fn test_auto_complete() {}

// #[test]
// fn test_search_fields() {}

// #[test]
// fn test_ordering() {}

// #[test]
// fn test_format() {}

// #[test]
// fn test_default_value() {}
