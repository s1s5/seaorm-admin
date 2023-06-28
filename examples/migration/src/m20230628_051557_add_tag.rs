use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tag::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tag::Name).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TagRelation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TagRelation::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TagRelation::TagId).integer().not_null())
                    .col(ColumnDef::new(TagRelation::AuthorId).integer().not_null())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-tagId")
                            .from_tbl(TagRelation::Table)
                            .from_col(TagRelation::TagId)
                            .to_tbl(Tag::Table)
                            .to_col(Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-authorId")
                            .from_tbl(TagRelation::Table)
                            .from_col(TagRelation::AuthorId)
                            .to_tbl(Author::Table)
                            .to_col(Author::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TagRelation::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Tag {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum TagRelation {
    Table,
    Id,
    TagId,
    AuthorId,
}

#[derive(Iden)]
enum Author {
    Table,
    Id,
}
