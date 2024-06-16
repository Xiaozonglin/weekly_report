use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use wr_database::{user, Database};

use crate::ResponseError;

pub async fn prepare_user_info(
    State(ref db): State<Database>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, ResponseError> {
    if let Some(user_id) = jar.get("id") {
        let user_id: i32 = user_id.value().parse()?;
        let user = user::get(&db.conn, user_id).await?;
        req.extensions_mut().insert(user);
        Ok(next.run(req).await)
    } else {
        Err(ResponseError::Unauthorized(
            "please login first".to_string(),
        ))
    }
}
