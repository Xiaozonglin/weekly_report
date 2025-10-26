use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::IntoResponse,
};
use tracing::warn;
use urlencoding::decode;
use wr_database::{user, Database};

use crate::ResponseError;

pub async fn prepare_user_info(
    State(ref db): State<Database>,
    header: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, ResponseError> {
    if let Some(email) = header.get("x-email") {
        let email = decode(email.to_str()?)?.to_string();
        let user = user::get_by_email(&db.conn, &email).await?;
        if let Some(user) = user {
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        } else {
            let Some(nickname) = header.get("x-nickname") else {
                warn!(
                    "x-email not found and no x-nickname header, original req: {:?}",
                    header
                );
                return Err(ResponseError::Unauthorized(
                    "please login first".to_string(),
                ));
            };
            let nickname = decode(nickname.to_str()?)?.to_string();
            let user = user::get_by_name(&db.conn, &nickname).await?;
            if let Some(user) = user {
                req.extensions_mut().insert(user);
                Ok(next.run(req).await)
            } else {
                warn!("user not found: {}, original req: {:?}", email, header);
                Err(ResponseError::Unauthorized(
                    "please login first".to_string(),
                ))
            }
        }
    } else {
        warn!("no x-email header found, original req: {:?}", header);
        Err(ResponseError::Unauthorized(
            "please login first".to_string(),
        ))
    }
}
