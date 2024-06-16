use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "Configs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub begin_week: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub skip_weeks: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
