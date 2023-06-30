use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(TagRelation::Table)
                    .name("author_tag_unique")
                    .col(TagRelation::AuthorId)
                    .col(TagRelation::TagId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(TagRelation::Table)
                    .name("author_tag_unique")
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum TagRelation {
    Table,
    TagId,
    AuthorId,
}
