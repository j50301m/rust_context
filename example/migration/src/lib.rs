use sea_orm_migration::{async_trait, MigrationTrait, MigratorTrait};

pub mod m20240816_021302_crate_hello_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20240816_021302_crate_hello_table::Migration)]
    }
}
