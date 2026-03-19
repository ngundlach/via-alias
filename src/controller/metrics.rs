use axum::{extract::State, response::IntoResponse};

use crate::AppContext;

#[utoipa::path(get,
    path = "/metrics",
    tag="metrics",
    summary = "Prometheus metrics",
    description = "Provides prometheus metrics",
    operation_id = "metrics",
    security(),
    responses(
        (status = StatusCode::OK),
))]
pub(crate) async fn metrics_handler(State(app_context): State<AppContext>) -> impl IntoResponse {
    app_context.metrics.render()
}
