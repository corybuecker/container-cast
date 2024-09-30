use axum::Router;
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::fmt;
mod webhook;
use webhook::router;

#[tokio::main]
async fn main() {
    initialize_tracing();

    let app = Router::new().nest("/", router()).layer(trace_layer());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn initialize_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .event_format(fmt::format().pretty())
        .init()
}

fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}
