use std::{net::IpAddr, time::Duration};

use axum::{
    body::Body,
    extract::{Query, Request, State},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use chrono::{Datelike, Utc};
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::{debug, debug_span, Span};
use wr_database::{report, user, Database};

use crate::{
    middleware::{auth, data, forwarded},
    traits::GlobalState,
    ResponseError,
};

pub async fn initialize(state: GlobalState) -> anyhow::Result<Router> {
    let api_router = construct_router(&state);
    let router = Router::new()
        .nest("/api", api_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let ip = forwarded::get_client_ip(request)
                        .unwrap_or(IpAddr::V4("0.0.0.0".parse().unwrap()));
                    debug_span!("http",
                        from = %ip.to_string(),
                        method = %request.method(),
                        uri = %request.uri().path(),
                    )
                })
                .on_request(())
                .on_response(|response: &Response, latency: Duration, _span: &Span| {
                    debug!("[{}] in {}ms", response.status(), latency.as_millis());
                }),
        )
        .with_state::<()>(state);
    Ok(router)
}

pub fn construct_router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/import", post(import_users))
        .route("/user", patch(modify_user))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::admin_required,
        ))
        .route("/user", get(get_user))
        .route("/report", get(get_report).post(handle_submit))
        .route("/self", get(get_self_info))
        .route("/ping", get(ping))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            data::prepare_user_info,
        ))
}

async fn ping() -> impl IntoResponse {
    "pong"
}

#[derive(Deserialize)]
struct UserQuery {
    pub id: Option<i32>,
    pub with_hidden: Option<bool>,
}

async fn import_users(
    State(ref db): State<Database>,
    Json(users): Json<Vec<user::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
    user::create_list(&db.conn, users).await?;
    Ok(())
}

async fn modify_user(
    State(ref db): State<Database>,
    Json(user): Json<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(Json(user::update(&db.conn, user).await?))
}

async fn get_user(
    State(ref db): State<Database>,
    Query(query): Query<UserQuery>,
) -> Result<impl IntoResponse, ResponseError> {
    match query {
        UserQuery {
            id: Some(id),
            with_hidden: _,
        } => {
            let user = user::get(&db.conn, id).await?;
            Ok(Json(user).into_response())
        }
        UserQuery {
            id: None,
            with_hidden,
        } => {
            let users = user::get_list(&db.conn, with_hidden.unwrap_or(false)).await?;
            Ok(Json(users).into_response())
        }
    }
}

#[derive(Deserialize)]
struct SubmitForm {
    pub content: String,
}

async fn handle_submit(
    State(ref db): State<Database>,
    Extension(user): Extension<user::Model>,
    Json(form): Json<SubmitForm>,
) -> Result<impl IntoResponse, ResponseError> {
    let date = Utc::now();
    if date.weekday() != chrono::Weekday::Sun {
        return Err(ResponseError::BadRequest("only Sunday".to_string()));
    }
    let week = date.year() * 10_000 + date.month() as i32 * 100 + date.day() as i32;
    let report = report::get(&db.conn, user.id, week).await?;
    if let Some(report) = report {
        Ok(Json(
            report::update(
                &db.conn,
                report::Model {
                    content: Some(form.content),
                    ..report
                },
            )
            .await?,
        ))
    } else {
        Ok(Json(
            report::create(&db.conn, user.id, week, form.content).await?,
        ))
    }
}

async fn get_self_info(
    Extension(user): Extension<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(Json(user))
}

#[derive(Deserialize)]
struct ReportQuery {
    pub user: Option<i32>,
    pub week: Option<i32>,
}

async fn get_report(
    State(ref db): State<Database>,
    Query(query): Query<ReportQuery>,
) -> Result<impl IntoResponse, ResponseError> {
    match query {
        ReportQuery {
            // return the report content
            user: Some(user),
            week: Some(week),
        } => {
            let report = report::get(&db.conn, user, week).await?;
            Ok(Json(report).into_response())
        }
        ReportQuery {
            // return user's report list
            user: Some(user),
            week: None,
        } => {
            let reports = report::get_user_list(&db.conn, user).await?;
            Ok(Json(reports).into_response())
        }
        ReportQuery {
            // return week's report list
            user: None,
            week: Some(week),
        } => {
            let reports = report::get_week_list(&db.conn, week).await?;
            Ok(Json(reports).into_response())
        }
        _ =>
        // return reports for index table
        {
            let reports = report::get_index_list(&db.conn).await?;
            let users = user::get_list(&db.conn, false).await?;
            Ok(Json((users, reports)).into_response())
        }
    }
}
