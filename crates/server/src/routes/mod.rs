use std::{net::IpAddr, time::Duration};

use axum::{
    body::Body,
    extract::{Query, Request, State},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use chrono::{Datelike, Duration as ChronoDuration, Utc, DateTime};
use serde::{Deserialize, Serialize};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{debug, debug_span, Span};
use wr_database::{report, user, Database};
use wr_database::report::ExModel;

use crate::{
    middleware::{auth, data, forwarded},
    traits::GlobalState,
    ResponseError,
};

pub async fn initialize(state: GlobalState) -> anyhow::Result<Router> {
    let api_router = construct_router(&state);
    let serve_dir = ServeDir::new(std::env::var("WR_STATIC")?)
        .precompressed_gzip()
        .not_found_service(ServeFile::new(format!(
            "{}/index.html",
            std::env::var("WR_STATIC")?
        )));
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
        .fallback_service(serve_dir)
        .with_state::<()>(state);
    Ok(router)
}

pub fn construct_router(state: &GlobalState) -> Router<GlobalState> {
    // public routes (no auth required)
    let public = Router::new().route("/{id}/feed/", get(get_user_feed));

    // protected routes (may apply middleware)
    // Admin-only routes: put under a small admin router that will be merged into protected
    let admin_router = Router::new()
        .route("/import", post(import_users))
        .route("/user", patch(modify_user))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::admin_required,
        ));

    // Protected routes: first construct routes (including merging admin routes),
    // then apply the prepare_user_info middleware so it attaches Extension<user::Model>
    // for all protected endpoints (including admin routes which still have their
    // own admin_required middleware).
    let protected = Router::new()
        .merge(admin_router)
        .route("/user", get(get_user))
        .route("/report", get(get_report).post(handle_submit))
    // like a report: POST /api/report/{id}/like
    .route("/report/{id}/like", post(like_report))
    // unlike a report: POST /api/report/{id}/unlike
    .route("/report/{id}/unlike", post(unlike_report))
        .route("/self", get(get_self_info))
        .route("/ping", get(ping))
    .route("/self/feed_token", get(get_or_create_feed_token).post(regenerate_feed_token))
        .route("/status", get(get_status))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            data::prepare_user_info,
        ));

    public.merge(protected)
}

async fn ping() -> impl IntoResponse {
    "pong"
}

#[derive(Serialize)]
struct StatusResponse {
    submitted: Vec<String>,
    pending: Vec<String>,
}

async fn get_status(State(ref db): State<Database>) -> Result<impl IntoResponse, ResponseError> {
    let users = user::get_list(&db.conn, false).await?;
    let now = Utc::now();
    let next_sunday = if now.weekday() != chrono::Weekday::Sun {
        now + ChronoDuration::days(6 - now.weekday().num_days_from_sunday() as i64)
    } else {
        now
    };
    let edge =
        next_sunday.year() * 10_000 + next_sunday.month() as i32 * 100 + next_sunday.day() as i32;
    let reports = report::get_week_list(&db.conn, edge).await?;
    let mut submitted = vec![];
    let mut pending = vec![];
    for user in users {
        let report = reports.iter().find(|r| r.author_id == user.id);
        if report.is_some() {
            submitted.push(user.name.clone());
        } else {
            pending.push(user.name.clone());
        }
    }
    Ok(Json(StatusResponse { submitted, pending }))
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
        return Err(ResponseError::BadRequest("Only Sunday".to_string()));
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

// DTOs used by server responses: keep date serialization consistent and
// expose `likes` as an array instead of the DB's JSON string.
#[derive(Serialize)]
struct ReportDto {
    pub id: i32,
    pub author_id: i32,
    pub week: i32,
    pub content: Option<String>,
    pub likes: Option<Vec<String>>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date: DateTime<Utc>,
}

#[derive(Serialize)]
struct ExReportDto {
    pub id: i32,
    pub author_id: i32,
    pub author_name: String,
    pub week: i32,
    pub content: Option<String>,
    pub likes: Option<Vec<String>>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date: DateTime<Utc>,
}

fn parse_likes_field(s: &Option<String>) -> Option<Vec<String>> {
    match s {
        Some(t) => serde_json::from_str::<Vec<String>>(t).ok(),
        None => None,
    }
}

fn model_to_dto(m: report::Model) -> ReportDto {
    ReportDto {
        id: m.id,
        author_id: m.author_id,
        week: m.week,
        content: m.content,
        likes: parse_likes_field(&m.likes),
        date: m.date,
    }
}

fn exmodel_to_dto(m: ExModel) -> ExReportDto {
    ExReportDto {
        id: m.id,
        author_id: m.author_id,
        author_name: m.author_name,
        week: m.week,
        content: m.content,
        likes: parse_likes_field(&m.likes),
        date: m.date,
    }
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
            let report = report::get_ex(&db.conn, user, week).await?;
            let dto = report.map(|r| exmodel_to_dto(r));
            Ok(Json(dto).into_response())
        }
        ReportQuery {
            // return user's report list
            user: Some(user),
            week: None,
        } => {
            let reports = report::get_user_list(&db.conn, user).await?;
            let dtos: Vec<ReportDto> = reports.into_iter().map(model_to_dto).collect();
            Ok(Json(dtos).into_response())
        }
        ReportQuery {
            // return week's report list
            user: None,
            week: Some(week),
        } => {
            let reports = report::get_week_list(&db.conn, week).await?;
            let dtos: Vec<ExReportDto> = reports.into_iter().map(exmodel_to_dto).collect();
            Ok(Json(dtos).into_response())
        }
        _ =>
        // return reports for index table
        {
            let reports = report::get_index_list(&db.conn).await?;
            let users = user::get_list(&db.conn, false).await?;
            let dtos: Vec<ReportDto> = reports.into_iter().map(model_to_dto).collect();
            Ok(Json((users, dtos)).into_response())
        }
    }
}

use uuid::Uuid;

#[derive(Deserialize)]
struct FeedQuery {
    token: Option<String>,
}

async fn get_user_feed(
    State(ref db): State<Database>,
    axum::extract::Path(id): axum::extract::Path<i32>,
    Query(query): Query<FeedQuery>,
) -> Result<impl IntoResponse, ResponseError> {
    let user = user::get(&db.conn, id).await?;
    let user = match user {
        Some(u) => u,
        None => return Err(ResponseError::NotFound("user not found".to_string())),
    };
    let reports = report::get_user_ex_list(&db.conn, id).await?;

    // Prefer explicit public URL from env for stability (WR_PUBLIC_URL), fall back to localhost
    let base = std::env::var("WR_PUBLIC_URL").unwrap_or_else(|_| "http://localhost".to_string());

    let feed = build_rss_feed(&user.name, user.id, &reports, &base);

    // check token: require token param and validate that token exists in DB
    let token = match query.token {
        Some(t) => t,
        None => return Err(ResponseError::Unauthorized("token required".to_string())),
    };
    let subscriber = user::get_by_feed_token(&db.conn, &token).await?;
    let subscriber = match subscriber {
        None => return Err(ResponseError::Unauthorized("invalid token".to_string())),
        Some(s) => s,
    };
    // Deny access for banned subscribers
    if subscriber.is_banned {
        return Err(ResponseError::Unauthorized("subscriber banned".to_string()));
    }

    // Record auth/logging event: who accessed whose feed and when. Do NOT log the token.
    tracing::info!(
        subscriber_id = subscriber.id,
        subscriber_name = %subscriber.name,
        author_id = user.id,
        author_name = %user.name,
        time = %Utc::now().to_rfc3339(),
        "rss feed access"
    );

    // Return with proper RSS content-type
    Ok((
        axum::http::StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "application/rss+xml; charset=utf-8"),
            (axum::http::header::HeaderName::from_static("referrer-policy"), "no-referrer"),
            (axum::http::header::HeaderName::from_static("cache-control"), "private, max-age=300"),
            (axum::http::header::HeaderName::from_static("x-content-type-options"), "nosniff"),
        ],
        feed,
    ))
}

async fn get_or_create_feed_token(
    State(ref db): State<Database>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
    // if user already has feed_token return it, otherwise create a new one and persist
    if let Some(token) = current_user.feed_token.clone() {
        return Ok(Json(serde_json::json!({ "token": token })));
    }
    let mut new_user = current_user.clone();
    let token = Uuid::new_v4().to_string();
    new_user.feed_token = Some(token.clone());
    let updated = user::update(&db.conn, new_user).await?;
    Ok(Json(serde_json::json!({ "token": updated.feed_token })))
}

// Regenerate (reset) the current user's feed token. Protected endpoint.
async fn regenerate_feed_token(
    State(ref db): State<Database>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
    let mut new_user = current_user.clone();
    let token = Uuid::new_v4().to_string();
    new_user.feed_token = Some(token.clone());
    let updated = user::update(&db.conn, new_user).await?;
    Ok(Json(serde_json::json!({ "token": updated.feed_token })))
}

// Like a report. Bodyless POST. Requirements:
// - authenticated user (provided by data::prepare_user_info middleware)
// - cannot like own report
// - cannot like twice (username unique)
async fn like_report(
    State(ref db): State<Database>,
    Extension(current_user): Extension<user::Model>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<impl IntoResponse, ResponseError> {
    // find report by id using database helper
    let r = report::get_by_id(&db.conn, id).await?;
    let r = match r {
        Some(v) => v,
        None => return Err(ResponseError::NotFound("report not found".to_string())),
    };

    tracing::info!(user = %current_user.name, report_id = id, "like_report invoked");

    if r.author_id == current_user.id {
        tracing::warn!(user = %current_user.name, report_id = id, "attempted to like own report");
        return Err(ResponseError::BadRequest("cannot like your own report".to_string()));
    }

    // parse likes JSON array
    let mut likes: Vec<String> = match r.likes.clone() {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => vec![],
    };

    if likes.iter().any(|n| n == &current_user.name) {
        return Err(ResponseError::BadRequest("already liked".to_string()));
    }

    likes.push(current_user.name.clone());
    let likes_s = serde_json::to_string(&likes)?;

    // update only likes column using DB helper to avoid clobbering other fields
    match report::update_likes_by_id(&db.conn, id, Some(likes_s.clone())).await {
        Ok(_) => {
            tracing::info!(user = %current_user.name, report_id = id, likes_count = likes.len(), "like_report success");
            Ok(Json(serde_json::json!({ "likes": likes })))
        }
            Err(e) => {
            tracing::error!(error = %e, user = %current_user.name, report_id = id, "like_report db error");
            Err(ResponseError::InternalServerError("failed to update likes".to_string(), e.to_string()))
        }
    }
}

async fn unlike_report(
    State(ref db): State<Database>,
    Extension(current_user): Extension<user::Model>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<impl IntoResponse, ResponseError> {
    let r = report::get_by_id(&db.conn, id).await?;
    let r = match r {
        Some(v) => v,
        None => return Err(ResponseError::NotFound("report not found".to_string())),
    };

    tracing::info!(user = %current_user.name, report_id = id, "unlike_report invoked");

    // cannot unlike your own report
    if r.author_id == current_user.id {
        tracing::warn!(user = %current_user.name, report_id = id, "attempted to unlike own report");
        return Err(ResponseError::BadRequest("cannot unlike your own report".to_string()));
    }

    let mut likes: Vec<String> = match r.likes.clone() {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => vec![],
    };

    // remove any occurrences of the current user name
    likes.retain(|n| n != &current_user.name);

    let likes_s = serde_json::to_string(&likes)?;

    match report::update_likes_by_id(&db.conn, id, Some(likes_s.clone())).await {
        Ok(_) => {
            tracing::info!(user = %current_user.name, report_id = id, likes_count = likes.len(), "unlike_report success");
            Ok(Json(serde_json::json!({ "likes": likes })))
        }
        Err(e) => {
            tracing::error!(error = %e, user = %current_user.name, report_id = id, "unlike_report db error");
            Err(ResponseError::InternalServerError("failed to update likes".to_string(), e.to_string()))
        }
    }
}

/// Build RSS 2.0 XML for given user and reports. Description is wrapped in CDATA and
/// any occurrence of `]]>` inside content is safely split.
pub fn build_rss_feed(
    user_name: &str,
    user_id: i32,
    reports: &[ExModel],
    base_url: &str,
) -> String {
    let mut items = String::new();
    for r in reports.iter() {
    // Use a concise, fixed title format instead of full content
    let item_title_text = format!("{}的第{}周周报", r.author_name, r.week);
    let title = html_escape::encode_text(&item_title_text);
        let pub_date = r.date.to_rfc2822();
    let link = format!("{}/user/{}/report/{}", base_url.trim_end_matches('/'), r.author_id, r.id);

        // Use HTML-escaped description (no CDATA)
    let desc = html_escape::encode_text(r.content.as_deref().unwrap_or("(no content)"));
        // Use a non-permalink guid that is unique: report-{id}-{timestamp}
        let guid_value = format!("report-{}-{}", r.id, r.date.timestamp());
        items.push_str(&format!(
            "<item><title>{}</title><link>{}</link><guid isPermaLink=\"false\">{}</guid><pubDate>{}</pubDate><description>{}</description></item>",
            title, link, guid_value, pub_date, desc
        ));
    }

    // include atom namespace and self link for better interoperability
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\"><channel><title>{}'s Reports</title><link>{}/user/{}</link><atom:link href=\"{}/user/{}/feed/\" rel=\"self\" type=\"application/rss+xml\" /><description>RSS feed for user {}</description>{}</channel></rss>",
        html_escape::encode_text(user_name), base_url.trim_end_matches('/'), user_id, base_url.trim_end_matches('/'), user_id, html_escape::encode_text(user_name), items
    )
}



