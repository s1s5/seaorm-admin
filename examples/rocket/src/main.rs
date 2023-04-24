use std::sync::Arc;
use rocket::routes;
use entity::{author, post, test_model};
use seaorm_admin::rocket_admin::*;
use seaorm_admin::{Admin, ModelAdmin};

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

#[derive(ModelAdmin, Default)]
#[model_admin(module = post, auto_complete = [Author])]
struct PostAdmin;

#[derive(ModelAdmin, Default)]
#[model_admin(module = test_model)]
struct TestAdmin;

#[tokio::main]
async fn main() -> std::result::Result<(), rocket::Error> {
    let connection = Arc::new(
        sea_orm::Database::connect(std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("Could not connect to database. Please set DATABASE_URL"),
    );

    let mut admin = Admin::new(connection, "");
    admin.add_model(AuthorAdmin);
    admin.add_model(PostAdmin);
    admin.add_model(TestAdmin);

    rocket::build()
        .manage(admin)
        .mount(
            "/",
            routes![
                index,
                list,
                get_create_template,
                create_model,
                get_update_template,
                update_model,
                get_delete_template,
                delete_model,
            ],
        )
        .launch()
        .await
        .map(|_| ())
}
