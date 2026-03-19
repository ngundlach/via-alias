// src/telemetry.rs
use metrics_exporter_prometheus::PrometheusBuilder;

pub fn init_metrics() -> metrics_exporter_prometheus::PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder")
}
