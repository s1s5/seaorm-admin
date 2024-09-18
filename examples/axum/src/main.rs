use axum::{extract::Extension, Router};
use entity::{author, post, tag, tag_relation, test_model};
use sea_orm::Set;
use seaorm_admin::{enum_field, inline_field, m2m_field, Admin, AdminBuilder, ModelAdmin};
use std::net::SocketAddr;
use std::sync::Arc;

fn format_author(model: &author::Model) -> String {
    format!("[CUSTOM] author[{}]({})", model.id, model.name)
}

fn get_initial_author() -> author::ActiveModel {
    author::ActiveModel {
        main_post_id: Set(None),
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
    initial_value = get_initial_author,
    form_fields = [
        inline_field("posts", post::Relation::Author.def(), true),
        m2m_field(
            "tags",
            tag_relation::Relation::Author.def(),
            tag_relation::Relation::Tag.def()
        ),
    ]
)]
struct AuthorAdmin;

#[derive(ModelAdmin, Default)]
#[model_admin(module = post, auto_complete=[Author])]
struct PostAdmin;

#[derive(ModelAdmin, Default)]
#[model_admin(module = test_model,
    form_fields = [
        enum_field(test_model::Column::EnumString, test_model::Category::iter()),
        enum_field(test_model::Column::EnumI32, test_model::Color::iter()),
    ],
)]
struct TestAdmin;

fn tag_display(model: &tag::Model) -> String {
    format!("Tag[{}] {}", model.id, model.name)
}

#[derive(ModelAdmin, Default)]
#[model_admin(module = tag, format=tag_display)]
struct TagAdmin;

#[derive(ModelAdmin, Default)]
#[model_admin(module = tag_relation)]
struct TagRelationAdmin;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let connection = Arc::new(
        sea_orm::Database::connect(std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("Could not connect to database. Please set DATABASE_URL"),
    );

    let admin = AdminBuilder::default()
        .add_model(AuthorAdmin)
        .add_model(PostAdmin)
        .add_model(TestAdmin)
        .add_model(TagAdmin)
        .add_model(TagRelationAdmin)
        .build(connection, "/admin")?;

    let app = Router::new()
        .nest(
            &format!("{}/", admin.sub_path()),
            seaorm_admin::axum_admin::get_router(),
        )
        .layer(Extension(Arc::new(admin)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("listening {:?}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
