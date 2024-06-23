use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::IntoResponse,
};
use urlencoding::decode;
use wr_database::{user, Database};

use crate::ResponseError;

pub async fn prepare_user_info(
    State(ref db): State<Database>,
    header: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, ResponseError> {
    if let Some(user_id) = header.get("x-nickname") {
        let nickname = decode(user_id.to_str()?)?.to_string();
        let user = user::get_by_name(&db.conn, &nickname).await?;
        if let Some(user) = user {
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        } else {
            Err(ResponseError::Unauthorized(
                "please login first".to_string(),
            ))
        }
    } else {
        Err(ResponseError::Unauthorized(
            "please login first".to_string(),
        ))
    }
}
