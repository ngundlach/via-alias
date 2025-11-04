mod controller;
mod data;
mod model;
use std::{env, error::Error, net::SocketAddr, sync::Arc};

use axum::{Router, http::StatusCode, routing::get};
use controller::redirect_controller;
use data::{RedirectRepo, SqliteService};
use sqlx::{Executor, migrate::MigrateDatabase};
use tokio::signal;

#[derive(Clone)]
struct AppState {
    db: Arc<dyn RedirectRepo + Send + Sync>,
}
#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        println!("Via-Aliases encountered an error: {}", e);
    }
}

async fn run_app() -> Result<(), Box<dyn Error>> {
    let db_file_env = env::var("VIA_ALIAS_DB");
    let db_file = db_file_env.unwrap_or_else(|_| "via-alias.db".to_string());
    if !sqlx::Sqlite::database_exists(&db_file).await? {
        sqlx::Sqlite::create_database(&db_file).await?;
    }
    let pool = sqlx::sqlite::SqlitePool::connect(&db_file).await?;
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS redirects (
              id INTEGER NOT NULL PRIMARY KEY,
              alias TEXT NOT NULL UNIQUE,
              url TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_redirects_alias ON redirects(alias);
        "#;
    pool.execute(create_table_query).await?;

    let dbservice = SqliteService::new(pool.clone());

    let app_state = AppState {
        db: Arc::new(dbservice),
    };

    let app = Router::new()
        .merge(redirect_controller::router(app_state))
        .route("/healthcheck", get(|| async { StatusCode::OK }));

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
