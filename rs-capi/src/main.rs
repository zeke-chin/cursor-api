mod handlers;
mod models;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
// use http::HeaderName as HttpHeaderName;
use tower_http::cors::{Any, CorsLayer};
mod hex_utils;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建CORS中间件
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 创建路由
    let app = Router::new()
        .route(
            "/v1/chat/completions",
            post(handlers::chat::chat_completions),
        )
        .route("/models", get(handlers::models::models))
        .route("/v1/models", get(handlers::models::models))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                // .on_request(|_request: &axum::http::Request<_>, _span: &tracing::Span| { info!("started processing request"); })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = %response.status(),
                            latency = ?latency,
                        );
                    },
                ),
        );

    // 启动服务器
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Server running on {}", addr);

    // 修改服务器启动代码
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
