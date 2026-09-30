use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait,
  PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use chrono::Utc;

use crate::{
  entity::todo::{self, ActiveModel, Column, Entity},
  error::AppError,
  models::todo::{CreateTodo, UpdateTodo},
};

pub type Db = DatabaseConnection;

pub async fn list(
  db: &Db,
  done: Option<bool>,
  page: u64,
  per_page: u64,
) -> Result<(Vec<todo::Model>, u64), AppError> {
  let mut cond = Column::DeletedAt.is_null();
  if let Some(done) = done {
    cond = cond.and(Column::Done.eq(done));
  }
  let finder = Entity::find().filter(cond);
  let total = finder.clone().count(db).await?;

  let items = finder
    .order_by_asc(Column::Id)
    .offset(page.saturating_sub(1) * per_page)
    .limit(per_page)
    .all(db)
    .await?;

  Ok((items, total))
}

pub async fn get(db: &Db, id: i32) -> Result<todo::Model, AppError> {
  Entity::find_by_id(id)
    .filter(Column::DeletedAt.is_null())
    .one(db)
    .await?
    .ok_or(AppError::todo_not_found(id as u64))
}

pub async fn create(db: &Db, req: CreateTodo) -> Result<todo::Model, AppError> {
  let am = ActiveModel {
    title: Set(req.title),
    description: Set(req.description),
    ..Default::default()
  };
  Ok(am.insert(db).await?)
}

pub async fn update(db: &Db, id: i32, req: UpdateTodo) -> Result<todo::Model, AppError> {
  let mut am: ActiveModel = get(db, id).await?.into();
  if let Some(title) = req.title {
    am.title = Set(title);
  }
  if let Some(description) = req.description {
    am.description = Set(Some(description));
  }
  if let Some(done) = req.done {
    am.done = Set(done);
  }
  Ok(am.update(db).await?)
}

/// 软删除：UPDATE deleted_at 而非 DELETE
pub async fn delete(db: &Db, id: i32) -> Result<(), AppError> {
  let mut am: ActiveModel = get(db, id).await?.into();
  am.deleted_at = Set(Some(Utc::now().into()));
  am.update(db).await?;
  Ok(())
}

pub async fn has_title(db: &Db, title: &str) -> Result<bool, AppError> {
  Ok(
    Entity::find()
      .filter(Column::DeletedAt.is_null())
      .filter(Column::Title.eq(title))
      .one(db)
      .await?
      .is_some(),
  )
}
