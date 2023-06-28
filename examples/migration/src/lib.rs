pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20230421_000833_add_test;
mod m20230628_051557_add_tag;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20230421_000833_add_test::Migration),
            Box::new(m20230628_051557_add_tag::Migration),
        ]
    }
}
