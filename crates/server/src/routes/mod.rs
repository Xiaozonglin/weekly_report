use std::{net::IpAddr, time::Duration};

use axum::{
    body::Body,
    extract::{Query, Request, State},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use chrono::{Datelike, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{debug, debug_span, Span};
use wr_database::{report, user, Database};

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
    let protected = Router::new()
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
        .route("/status", get(get_status));

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

async fn get_user_feed(
    State(ref db): State<Database>,
    axum::extract::Path(id): axum::extract::Path<i32>,
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

    // Return with proper RSS content-type
    Ok((
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        feed,
    ))
}

/// Build RSS 2.0 XML for given user and reports. Description is wrapped in CDATA and
/// any occurrence of `]]>` inside content is safely split.
pub fn build_rss_feed(
    user_name: &str,
    user_id: i32,
    reports: &Vec<wr_database::report::ExModel>,
    base_url: &str,
) -> String {
    let mut items = String::new();
    for r in reports.iter() {
        let title_text = r.content.as_ref().map(|s| s.as_str()).unwrap_or("(no content)");
        let title = html_escape::encode_text(title_text);
        let pub_date = r.date.to_rfc2822();
    let link = format!("{}/user/{}/report/{}", base_url.trim_end_matches('/'), r.author_id, r.id);

        // Use HTML-escaped description (no CDATA)
        let desc = html_escape::encode_text(title_text);
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



