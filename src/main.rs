mod controller;
mod data;
mod model;
mod service;
use std::{env, error::Error, net::SocketAddr, sync::Arc};

use axum::{Router, http::StatusCode, routing::get};
use sqlx::migrate::MigrateDatabase;
use tokio::signal;

use crate::{
    controller::redirect,
    data::RedirectRepoSqliteImpl,
    service::{RedirectService, RedirectServiceImpl},
};

#[derive(Clone)]
struct AppState {
    redirect_service: Arc<dyn RedirectService + Send + Sync>,
}
#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Via-Alias encountered an error: {}", e);
    }
}

async fn run_app() -> Result<(), Box<dyn Error>> {
    println!("Starting Via-Alias");
    let db_file_env = env::var("VIA_ALIAS_DB");
    let db_file = db_file_env.unwrap_or_else(|_| "via-alias.db".to_string());
    if !sqlx::Sqlite::database_exists(&db_file).await? {
        sqlx::Sqlite::create_database(&db_file).await?;
    }
    let pool = sqlx::sqlite::SqlitePool::connect(&db_file).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let redirect_repo = RedirectRepoSqliteImpl::new(pool.clone());
    let redirect_service = RedirectServiceImpl::new(redirect_repo);

    let app_state = AppState {
        redirect_service: Arc::new(redirect_service),
    };

    let app = Router::new()
        .merge(redirect::router())
        .route("/healthcheck", get(|| async { StatusCode::OK }))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 6789));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
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
        _ = ctrl_c => {println!("\nReceived ctrl+c event")},
        _ = terminate => {println!("Received termination signal")},
    }
}
