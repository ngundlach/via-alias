#![forbid(unsafe_code)]
#![deny(clippy::correctness)]
#![deny(clippy::suspicious)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
#![warn(clippy::style)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

use std::{env, error::Error, fs::read_to_string, net::SocketAddr, sync::Arc, time::Duration};

use axum::{Router, routing::get};
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase};
use tokio::signal;
use utoipa::OpenApi;

use crate::{
    controller::{admin, health_check, login, redirect, user},
    data::{RedirectRepoSqliteImpl, UserRegistrationTokenInMemoryImpl, UserRepoSqliteImpl},
    service::{
        LoginService, LoginServiceImpl, RedirectService, RedirectServiceImpl, UserService,
        UserServiceImpl,
    },
};

mod api_doc;
mod controller;
mod data;
mod middleware;
mod model;
mod service;
mod telemetry;

#[derive(Clone)]
struct AppContext {
    app_config: AppConfig,
    redirect_service: Arc<dyn RedirectService + Send + Sync>,
    login_service: Arc<dyn LoginService + Send + Sync>,
    user_service: Arc<dyn UserService + Send + Sync>,
    metrics: PrometheusHandle,
}
#[derive(Clone)]
struct AppConfig {
    port: u16,
    db_location: String,
    jwt_config: JwtConfig,
    reg_token_ttl: u64,
}
#[derive(Clone)]
struct JwtConfig {
    secret: String,
    alg: jsonwebtoken::Algorithm,
    ttl: u64,
}

fn create_app_context(pool: &Pool<Sqlite>, app_config: AppConfig) -> AppContext {
    let redirect_repo = Arc::new(RedirectRepoSqliteImpl::new(pool.clone()));
    let user_repo = Arc::new(UserRepoSqliteImpl::new(pool.clone()));
    let user_registration_token_repo = Arc::new(UserRegistrationTokenInMemoryImpl::new());
    user_registration_token_repo.start_cleanup(Duration::from_hours(1));
    let redirect_service = RedirectServiceImpl::new(redirect_repo);
    let user_service = UserServiceImpl::new(user_repo.clone(), user_registration_token_repo);
    let login_service = LoginServiceImpl::new(user_repo);
    let metrics = telemetry::init_metrics();
    AppContext {
        app_config,
        redirect_service: Arc::new(redirect_service),
        login_service: Arc::new(login_service),
        user_service: Arc::new(user_service),
        metrics,
    }
}

fn read_secret(name: &str) -> Result<String, std::io::Error> {
    read_to_string(format!("/run/secrets/{name}")).map(|s| s.trim().to_string())
}

fn generate_app_config() -> Result<AppConfig, Box<dyn Error>> {
    const JWT_SECRET_ENV: &str = "VIA_ALIAS_JWT_SECRET";
    const JWT_TTL: &str = "VIA_ALIAS_JWT_TTL";
    const PORT_ENV: &str = "VIA_ALIAS_PORT";
    const DB_LOC_ENV: &str = "VIA_ALIAS_DB";
    const REG_TOKEN_TTL: &str = "VIA_ALIAS_REG_TOKEN_TTL";
    let secret = read_secret(JWT_SECRET_ENV)
        .or_else(|_| env::var(JWT_SECRET_ENV))
        .map_err(|_| format!("{JWT_SECRET_ENV} is not set"))?;

    let port: u16 = env::var(PORT_ENV)
        .unwrap_or_else(|_| "6789".to_owned())
        .parse()
        .map_err(|_| format!("{PORT_ENV} is not a valid port number"))?;

    let jwt_ttl = env::var(JWT_TTL)
        .unwrap_or_else(|_| "900".to_owned())
        .parse()
        .map_err(|_| format!("{JWT_TTL} is not a valid value"))?;

    let db_location = env::var(DB_LOC_ENV)
        .unwrap_or_else(|_| "via-alias.db".to_owned())
        .parse()
        .map_err(|_| format!("{DB_LOC_ENV} is not a valid value"))?;

    let reg_token_ttl: u64 = env::var(REG_TOKEN_TTL)
        .unwrap_or_else(|_| "1800".to_owned())
        .parse()
        .map_err(|_| format!("{REG_TOKEN_TTL} is not a valid value"))?;

    let jwt_config = JwtConfig {
        secret,
        alg: jsonwebtoken::Algorithm::HS512,
        ttl: jwt_ttl,
    };

    Ok(AppConfig {
        port,
        db_location,
        jwt_config,
        reg_token_ttl,
    })
}

pub fn get_api_doc() -> Result<String, Box<dyn Error>> {
    let doc = api_doc::ApiDoc::openapi().to_pretty_json()?;
    Ok(doc)
}

fn create_router(context: AppContext) -> Router {
    Router::new()
        .merge(redirect::router())
        .merge(user::protected_user_management_router())
        .merge(admin::router())
        .layer(axum::middleware::from_fn_with_state(
            context.clone(),
            middleware::auth_middleware,
        ))
        .merge(user::user_router())
        .merge(login::router())
        .route("/{alias}", get(redirect::follow_redirect_handler))
        .route("/metrics", get(controller::metrics::metrics_handler))
        .with_state(context)
        .merge(api_doc::api_doc_router())
        .route("/healthcheck", get(health_check::health_check_handler))
        .layer(axum::middleware::from_fn(middleware::track_metrics))
}

pub async fn run_app() -> Result<(), Box<dyn Error>> {
    println!("Starting Via-Alias");
    let app_config = generate_app_config()?;
    if !sqlx::Sqlite::database_exists(&app_config.db_location).await? {
        sqlx::Sqlite::create_database(&app_config.db_location).await?;
    }
    let pool = sqlx::sqlite::SqlitePool::connect(&app_config.db_location).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = create_app_context(&pool, app_config);
    app_state.user_service.create_admin_first_start().await?;

    let port = app_state.app_config.port;
    let app = create_router(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Listening on port {port}...");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    println!("Closing database connection");
    pool.close().await;
    println!("Terminating...");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {println!("\nReceived ctrl+c event")},
        () = terminate => {println!("Received termination signal")},
    }
}
