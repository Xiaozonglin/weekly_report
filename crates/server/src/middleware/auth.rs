use axum::{extract::Request, middleware::Next, response::IntoResponse, Extension};
use wr_database::user;

use crate::ResponseError;

pub async fn admin_required(
    Extension(user): Extension<user::Model>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, ResponseError> {
    if user.is_admin {
        Ok(next.run(req).await)
    } else {
        Err(ResponseError::Forbidden(
            "admin required".to_string(),
            "only admin can access this resource".to_string(),
        ))
    }
}
