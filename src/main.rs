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

mod controller;
mod data;
mod middleware;
mod model;
mod service;
use axum::{Router, http::StatusCode, routing::get};
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase};
use std::{env, error::Error, fs::read_to_string, net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;

use crate::{
    controller::{admin, login, redirect, user},
    data::{RedirectRepoSqliteImpl, UserRegistrationTokenInMemoryImpl, UserRepoSqliteImpl},
    service::{
        LoginService, LoginServiceImpl, RedirectService, RedirectServiceImpl, UserService,
        UserServiceImpl,
    },
};

#[derive(Clone)]
struct AppContext {
    app_config: AppConfig,
    redirect_service: Arc<dyn RedirectService + Send + Sync>,
    login_service: Arc<dyn LoginService + Send + Sync>,
    user_service: Arc<dyn UserService + Send + Sync>,
}
#[derive(Clone)]
struct AppConfig {
    port: u16,
    db_location: String,
    jwt_config: JwtConfig,
}
#[derive(Clone)]
struct JwtConfig {
    jwt_secret: String,
    jwt_alg: jsonwebtoken::Algorithm,
    jwt_ttl: u64,
}

fn create_app_context(
    pool: &Pool<Sqlite>,
    app_config: AppConfig,
) -> Result<AppContext, Box<dyn Error>> {
    let redirect_repo = Arc::new(RedirectRepoSqliteImpl::new(pool.clone()));
    let user_repo = Arc::new(UserRepoSqliteImpl::new(pool.clone()));
    let user_registration_token_repo = Arc::new(UserRegistrationTokenInMemoryImpl::new());
    user_registration_token_repo.start_cleanup(Duration::from_hours(1));
    let redirect_service = RedirectServiceImpl::new(redirect_repo);
    let user_service = UserServiceImpl::new(user_repo.clone(), user_registration_token_repo);
    let login_service = LoginServiceImpl::new(user_repo);
    let app_context = AppContext {
        redirect_service: Arc::new(redirect_service),
        login_service: Arc::new(login_service),
        user_service: Arc::new(user_service),
        app_config,
    };
    Ok(app_context)
}

fn generate_app_config() -> Result<AppConfig, Box<dyn Error>> {
    const JWT_SECRET_ENV: &str = "VIA_ALIAS_JWT_SECRET";
    const JWT_TTL: &str = "VIA_ALIAS_JWT_TTL";
    const PORT_ENV: &str = "VIA_ALIAS_PORT";
    const DB_LOC_ENV: &str = "VIA_ALIAS_DB";
    let jwt_secret = read_secret(JWT_SECRET_ENV)
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

    let jwt_config = JwtConfig {
        jwt_secret,
        jwt_alg: jsonwebtoken::Algorithm::HS512,
        jwt_ttl,
    };

    Ok(AppConfig {
        port,
        jwt_config,
        db_location,
    })
}

fn read_secret(name: &str) -> Result<String, std::io::Error> {
    read_to_string(format!("/run/secrets/{name}")).map(|s| s.trim().to_string())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Via-Alias encountered an error: {e}");
    }
}

async fn run_app() -> Result<(), Box<dyn Error>> {
    println!("Starting Via-Alias");
    let app_config = generate_app_config()?;
    if !sqlx::Sqlite::database_exists(&app_config.db_location).await? {
        sqlx::Sqlite::create_database(&app_config.db_location).await?;
    }
    let pool = sqlx::sqlite::SqlitePool::connect(&app_config.db_location).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = create_app_context(&pool, app_config)?;
    app_state.user_service.create_admin_first_start().await?;

    let port = app_state.app_config.port;
    let app = Router::new()
        .merge(redirect::router())
        .merge(user::protected_user_management_router())
        .merge(admin::router())
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth_middleware,
        ))
        .merge(user::user_management_router())
        .merge(login::router())
        .route("/{alias}", get(redirect::get_redirect_handler))
        .route("/healthcheck", get(|| async { StatusCode::OK }))
        .with_state(app_state);

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
