use std::{error::Error, net::SocketAddr, sync::Arc};

mod app;
mod auth;
mod config;
mod features;
mod models;
mod repositories;

use app::create_app;
use config::get_config;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tokio::net::TcpListener;
use tracing::{info, span};
use tracing_subscriber::{Registry, prelude::*};

const SERVICE_NAME: &str = "pos_backend";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let resource = Resource::builder().with_service_name(SERVICE_NAME).build();
    let exporter = SpanExporter::builder().with_http().build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    let tracer = provider.tracer(SERVICE_NAME);
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default().with(telemetry);

    #[cfg(debug_assertions)]
    let subscriber = {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_level(true);
        subscriber.with(fmt_layer)
    };

    tracing::subscriber::set_global_default(subscriber)?;

    let root = span!(tracing::Level::INFO, "app_start");
    let _enter = root.enter();
    info!("server is running in {}", get_config().address);

    let app = create_app().await?;
    let listener = TcpListener::bind(get_config().address.as_str()).await?;

    let provider = Arc::new(provider);
    let shutdown_provider = provider.clone();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    shutdown_provider.shutdown()?;
    Ok(())
}

/// Waits for a Ctrl+C or SIGTERM signal to trigger graceful shutdown.
///
/// # Panics
///
/// Panics if the OS signal handlers cannot be installed — the server cannot
/// shut down gracefully without them.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
