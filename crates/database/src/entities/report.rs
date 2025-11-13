use chrono::{serde::ts_seconds, DateTime, Datelike, Duration, Utc};
use sea_orm::{
    entity::prelude::*, ActiveValue, FromQueryResult, IntoActiveModel, JoinType, QuerySelect,
};
use sea_orm::JsonValue;
use sea_orm::{DatabaseBackend, DatabaseConnection, Statement};
use serde::{Deserialize, Serialize};

use crate::user;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "Reports")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub author_id: i32,
    pub week: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub content: Option<String>,
    // Use SeaORM's JsonValue for JSON column storage
    #[sea_orm(column_type = "Json", nullable)]
    pub likes: Option<JsonValue>,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, FromQueryResult)]
pub struct ExModel {
    pub id: i32,
    pub author_id: i32,
    pub author_name: String,
    pub week: i32,
    pub content: Option<String>,
    // Keep the same JSON type for joined model
    pub likes: Option<JsonValue>,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Author,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Author.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get<C>(db: &C, user_id: i32, week: i32) -> Result<Option<Model>, DbErr>
where
    C: ConnectionTrait,
{
    Entity::find()
        .filter(Column::AuthorId.eq(user_id))
        .filter(Column::Week.eq(week))
        .one(db)
        .await
}

pub async fn get_ex<C>(db: &C, user_id: i32, week: i32) -> Result<Option<ExModel>, DbErr>
where
    C: ConnectionTrait,
{
    Entity::find()
        .join(JoinType::InnerJoin, Relation::Author.def())
        .column_as(user::Column::Name, "author_name")
        .filter(Column::AuthorId.eq(user_id))
        .filter(Column::Week.eq(week))
        .into_model()
        .one(db)
        .await
}

pub async fn get_user_list<C>(db: &C, user_id: i32) -> Result<Vec<Model>, DbErr>
where
    C: ConnectionTrait,
{
    Entity::find()
        .select_only()
        .columns(vec![
            Column::Week,
            Column::Id,
            Column::AuthorId,
            Column::Date,
        ])
        .filter(Column::AuthorId.eq(user_id))
        .all(db)
        .await
}

pub async fn get_week_list<C>(db: &C, week: i32) -> Result<Vec<ExModel>, DbErr>
where
    C: ConnectionTrait,
{
    Entity::find()
        .select_only()
        .columns(vec![
            Column::Week,
            Column::Id,
            Column::AuthorId,
            Column::Date,
        ])
        .join(JoinType::InnerJoin, Relation::Author.def())
        .column_as(user::Column::Name, "author_name")
        .filter(Column::Week.eq(week))
        .into_model()
        .all(db)
        .await
}

pub async fn get_user_ex_list<C>(db: &C, user_id: i32) -> Result<Vec<ExModel>, DbErr>
where
    C: ConnectionTrait,
{
    Entity::find()
        .join(JoinType::InnerJoin, Relation::Author.def())
        .column_as(user::Column::Name, "author_name")
        .filter(Column::AuthorId.eq(user_id))
        .into_model()
        .all(db)
        .await
}

pub async fn get_index_list<C>(db: &C) -> Result<Vec<Model>, DbErr>
where
    C: ConnectionTrait,
{
    let now = Utc::now();
    let next_sunday = if now.weekday() != chrono::Weekday::Sun {
        now + Duration::days(6 - now.weekday().num_days_from_sunday() as i64)
    } else {
        now
    };
    let six_weeks_ago = next_sunday - Duration::weeks(6);
    let edge = six_weeks_ago.year() * 10_000
        + six_weeks_ago.month() as i32 * 100
        + six_weeks_ago.day() as i32;
    Entity::find()
        .select_only()
        .columns(vec![
            Column::Week,
            Column::Id,
            Column::AuthorId,
            Column::Date,
        ])
        .join(JoinType::InnerJoin, Relation::Author.def())
        .filter(user::Column::IsHidden.eq(false))
        .filter(Column::Week.gt(edge))
        .all(db)
        .await
}

pub async fn create<C>(db: &C, user_id: i32, week: i32, content: String) -> Result<Model, DbErr>
where
    C: ConnectionTrait,
{
    let model = Model {
        id: 0,
        author_id: user_id,
        week,
        content: Some(content),
        // default to empty JSON array to guarantee valid JSON stored
        likes: Some(JsonValue::Array(vec![])),
        date: Utc::now(),
    };
    let model = model.into_active_model();
    let model = ActiveModel {
        id: ActiveValue::NotSet,
        ..model
    };
    model.insert(db).await
}

pub async fn get_by_id<C>(db: &C, id: i32) -> Result<Option<Model>, DbErr>
where
    C: ConnectionTrait,
{
    Entity::find_by_id(id).one(db).await
}

pub async fn update<C>(db: &C, model: Model) -> Result<Model, DbErr>
where
    C: ConnectionTrait,
{
    // Convert the incoming Model into an ActiveModel and update only the fields
    // that should change. Using `reset_all()` here previously caused optional
    // fields like `content` to become NotSet, which leads to clearing them in
    // the DB on update. Instead, preserve the model's fields and only set the
    // `date` to now.
    let mut am = model.into_active_model();
    am.date = ActiveValue::Set(Utc::now());
    am.update(db).await
}

/// Update only the likes column for a given report id. This avoids touching
/// other fields and is useful for concurrent-like operations.
pub async fn update_likes_by_id<C>(
    db: &C,
    id: i32,
    likes: Option<Vec<String>>,
) -> Result<Model, DbErr>
where
    C: ConnectionTrait,
{
    // Use ActiveModel update for a single row instead of update_many
    let am = ActiveModel {
        id: ActiveValue::Set(id),
        likes: ActiveValue::Set(likes.map(|v| serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect()))),
        date: ActiveValue::Set(Utc::now()),
        ..Default::default()
    };
    am.update(db).await
}