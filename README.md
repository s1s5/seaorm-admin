# admin library for sea-orm
`seaorm-admin` is a library for creating admin web-ui for sea-orm.

## Features
- macro for sea-orm Model

## Installation
```toml
seaorm-admin = { git = "https://github.com/s1s5/seaorm-admin" }
```

### required
```toml
sea-orm = "^0"
async-trait = "^0"
```

## Run Example
- run postgres
```shell
$ docker run --rm --tmpfs=/pgtmpfs \
    -e PGDATA=/pgtmpfs -e POSTGRES_HOST_AUTH_METHOD=trust \
    --name postgres -p  15432:5432 postgres
```
- in another shell
```shell
$ export DATABASE_URL=postgres://postgres:postgres@localhost:15432/postgres
$ cd examples/migrations
$ cargo run
$ cd ../rocket
$ cargo run
# access http://localhost:8000/admin/
```

## Usage
example entity file.
```Rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "author")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub main_post_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::post::Entity",
        from = "Column::MainPostId",
        to = "super::post::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Post,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### simple usage
```Rust
use entity::author;

#[derive(ModelAdmin, Default)]
#[model_admin(module = author)
struct AuthorAdmin;
```

### full option
```Rust
use entity::author;

fn format_author(model: &author::Model) -> String {
    format!("author[{}]({})", model.id, model.name)
}

fn get_initial_author() -> author::ActiveModel {
    author::ActiveModel {
        ..Default::default()
    }
}

#[derive(ModelAdmin, Default)]
#[model_admin(
    module = author,
    list_display = [Id, Name],
    fields = [Id, Name, MainPostId],
    auto_complete = [Post],
    search_fields = [Id, Name],
    ordering = [(Id, Desc)],
    format = format_author,
    initial_value = get_initial_author
)]
struct AuthorAdmin;
```

### use with Rocket
```Rust
use entity::author;
use seaorm_admin::rocket_admin::get_admin_routes;
use seaorm_admin::{Admin, ModelAdmin};
use std::sync::Arc;

#[derive(ModelAdmin, Default)]
#[model_admin(module = author)
struct AuthorAdmin;

#[tokio::main]
async fn main() -> std::result::Result<(), rocket::Error> {
  let connection = Arc::new(
      sea_orm::Database::connect(std::env::var("DATABASE_URL").unwrap())
          .await
          .expect("Could not connect to database. Please set DATABASE_URL"),
  );
  
  let mut admin = Admin::new(connection, "/admin");
  admin.add_model(AuthorAdmin);
  
  rocket::build()
      .mount(admin.sub_path(), get_admin_routes())
      .manage(admin)
      .launch()
      .await
      .map(|_| ())
}
```

## options
- `module`
required, set path of entity module.
- `list_display`
list of Columns. These fields are used in list view.
- `fields`
list of Columns. These fields are displayed in form view.
- `auto_complete`
list of Relations. These relations are used in form view.
- `search_fields`
list of Columns. These fields are used when searching in list view.
- `ordering`
list of (Column, Asc | Desc). used in list view.
- `format`
identity for Model -> String function. used in auto_complete
- `initial_value`
identity for the function returns AtctiveModel. used when creating, and some times called for create form.

## null handling when set empty string in the form
| nullable | field | db-value |
| -------- | ----- | -------- |
| nullable | Char, String | null |
| non-null | Char, String | "" |
| nullable | other | null |
| non-null | other | error |
