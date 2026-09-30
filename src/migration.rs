use sea_orm::{DeriveMigrationName, entity::prelude::async_trait};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Todos::Table)
          .col(pk_auto(Todos::Id))
          .col(string(Todos::Title))
          .col(string_null(Todos::Description))
          .col(boolean(Todos::Done).default(false))
          .col(timestamp_with_time_zone(Todos::CreatedAt).default(Expr::current_timestamp()))
          .col(timestamp_with_time_zone_null(Todos::DeletedAt)) // 软删除
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum Todos {
  Table,
  Id,
  Title,
  Description,
  Done,
  CreatedAt,
  DeletedAt,
}

pub struct Migrator;
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(Migration)]
  }
}
