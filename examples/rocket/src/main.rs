use entity::{author, post, test_model};
use sea_orm::Set;
use seaorm_admin::rocket_admin::get_admin_routes;
use seaorm_admin::{Admin, ModelAdmin};
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
    initial_value = get_initial_author
)]
struct AuthorAdmin;

#[derive(ModelAdmin, Default)]
#[model_admin(module = post, auto_complete=[Author])]
struct PostAdmin;

#[derive(ModelAdmin, Default)]
#[model_admin(module = test_model)]
struct TestAdmin;

#[tokio::main]
async fn main() -> std::result::Result<(), rocket::Error> {
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

    rocket::build()
        .mount(admin.sub_path(), get_admin_routes())
        .manage(admin)
        .launch()
        .await
        .map(|_| ())
}
