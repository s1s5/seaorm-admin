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
```

### full option
```Rust
use entity::author;

fn format_author(model: &author::Model) -> String {
    format!("author[{}]({})", model.id, model.name)
}

fn get_default_author() -> author::ActiveModel {
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
    default_value = get_default_author
)]
struct AuthorAdmin;
```

### options
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
- `default_value`
identity for the function returns AtctiveModel. used when creating.