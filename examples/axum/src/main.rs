use axum::{extract::Extension, Router};
use entity::{author, post, test_model};
use sea_orm::Set;
use seaorm_admin::{Admin, EnumWidget, ModelAdmin};
use std::net::SocketAddr;
use std::sync::Arc;

fn format_author(model: &author::Model) -> String {
    format!("author[{}]({})", model.id, model.name)
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
)]
struct AuthorAdmin;

#[derive(ModelAdmin, Default)]
#[model_admin(module = post, auto_complete=[Author])]
struct PostAdmin;

#[derive(ModelAdmin, Default)]
#[model_admin(module = test_model,
    widgets = [
        (EnumString, EnumWidget::from_enum(test_model::Category::iter())),
        (EnumI32, EnumWidget::from_enum(test_model::Color::iter())),
    ],
)]
struct TestAdmin;

#[tokio::main]
async fn main() -> std::result::Result<(), hyper::Error> {
    env_logger::init();

    let connection = Arc::new(
        sea_orm::Database::connect(std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("Could not connect to database. Please set DATABASE_URL"),
    );

    let mut admin = Admin::new(connection, "/admin");
    admin.add_model(AuthorAdmin);
    admin.add_model(PostAdmin);
    admin.add_model(TestAdmin);

    let app = Router::new()
        .nest(
            &format!("{}/", admin.sub_path()),
            seaorm_admin::axum_admin::get_router(),
        )
        .layer(Extension(Arc::new(admin)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
}
