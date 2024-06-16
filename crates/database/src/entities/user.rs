use sea_orm::{entity::prelude::*, ActiveValue, IntoActiveModel};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "Users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub email: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub direction: Option<String>,
    pub level: i32,
    pub is_banned: bool,
    pub is_hidden: bool,
    pub is_admin: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::report::Entity")]
    Report,
}

impl Related<super::report::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Report.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get<C>(db: &C, user_id: i32) -> Result<Option<Model>, DbErr>
where
    C: ConnectionTrait,
{
    Entity::find_by_id(user_id).one(db).await
}

pub async fn get_list<C>(db: &C, with_hidden: bool) -> Result<Vec<Model>, DbErr>
where
    C: ConnectionTrait,
{
    if with_hidden {
        Entity::find().all(db).await
    } else {
        Entity::find()
            .filter(Column::IsHidden.eq(false))
            .all(db)
            .await
    }
}

pub async fn create<C>(db: &C, model: Model) -> Result<Model, DbErr>
where
    C: ConnectionTrait,
{
    let user = ActiveModel {
        id: ActiveValue::NotSet,
        ..model.into_active_model().reset_all()
    };
    user.insert(db).await
}

pub async fn create_list<C>(db: &C, models: Vec<Model>) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let users = models
        .into_iter()
        .map(|model| ActiveModel {
            id: ActiveValue::NotSet,
            ..model.into_active_model().reset_all()
        })
        .collect::<Vec<_>>();
    Entity::insert_many(users).exec(db).await?;
    Ok(())
}

pub async fn update<C>(db: &C, model: Model) -> Result<Model, DbErr>
where
    C: ConnectionTrait,
{
    let user = ActiveModel {
        id: ActiveValue::Unchanged(model.id),
        ..model.into_active_model().reset_all()
    };
    user.update(db).await
}
