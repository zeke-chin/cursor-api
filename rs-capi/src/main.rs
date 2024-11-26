use axum::body::Body;
use axum::extract::Request;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use futures::{
    channel::mpsc,
    stream::{Stream, StreamExt},
    SinkExt,
};
use std::error::Error;
use tower_http::trace::TraceLayer;
// use http::HeaderName as HttpHeaderName;
use regex::Regex;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{convert::Infallible, time::Duration};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;
mod hex_utils;
use hex_utils::{chunk_to_utf8_string, string_to_hex};

// 定义请求模型
#[derive(Debug, Deserialize)]
struct Message {
    role: String,
    #[serde(deserialize_with = "deserialize_content")]
    content: Vec<ContentPart>,
}

// 添加一个辅助枚举
#[derive(Deserialize)]
#[serde(untagged)]
enum SingleOrVec<T> {
    Single(T),
    Vec(Vec<T>),
}

// 新增一个字符串或ContentPart的枚举
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ContentItem {
    String(String),
    Part(ContentPart),
}

// 新的反序列化函数
fn deserialize_content<'de, D>(deserializer: D) -> Result<Vec<ContentPart>, D::Error>
where
    D: Deserializer<'de>,
{
    // 首先尝试作为字符串反序列化
    let content = SingleOrVec::<ContentItem>::deserialize(deserializer)?;
    Ok(match content {
        SingleOrVec::Single(item) => match item {
            ContentItem::String(s) => vec![ContentPart::Text { text: s }],
            ContentItem::Part(p) => vec![p],
        },
        SingleOrVec::Vec(items) => items
            .into_iter()
            .map(|item| match item {
                ContentItem::String(s) => ContentPart::Text { text: s },
                ContentItem::Part(p) => p,
            })
            .collect(),
    })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Deserialize)]
struct ImageUrl {
    url: String,
}

impl std::fmt::Display for ContentPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentPart::Text { text } => write!(f, "{}", text),
            ContentPart::ImageUrl { image_url } => write!(f, "[Image: {}]", image_url.url),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(default)]
    stream: bool,
}

// 定义响应模型
#[derive(Debug, Serialize)]
struct ChatResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Serialize)]
struct Choice {
    index: i32,
    message: ResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
struct ResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Debug, Serialize)]
struct StreamResponse {
    id: String,
    object: String,
    created: i64,
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Serialize)]
struct StreamChoice {
    index: i32,
    delta: Delta,
}

#[derive(Debug, Serialize)]
struct Delta {
    content: String,
}

async fn process_stream(
    chunks: Vec<Bytes>,
) -> impl Stream<Item = Result<Event, Infallible>> + Send {
    let (mut tx, rx) = mpsc::channel(100);
    let response_id = format!("chatcmpl-{}", Uuid::new_v4());

    tokio::spawn(async move {
        for chunk in chunks {
            let text = chunk_to_utf8_string(&chunk);
            if !text.is_empty() {
                let text = text.trim();
                let text = if let Some(idx) = text.find("<|END_USER|>") {
                    text[idx + "<|END_USER|>".len()..].trim()
                } else {
                    text
                };

                let text = if !text.is_empty() && text.chars().next().unwrap().is_alphabetic() {
                    text[1..].trim()
                } else {
                    text
                };

                let re = Regex::new(r"[\x00-\x1F\x7F]").unwrap();
                let text = re.replace_all(text, "");

                if !text.is_empty() {
                    let response = StreamResponse {
                        id: response_id.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created: chrono::Utc::now().timestamp(),
                        choices: vec![StreamChoice {
                            index: 0,
                            delta: Delta {
                                content: text.to_string(),
                            },
                        }],
                    };

                    let json_data = serde_json::to_string(&response).unwrap();
                    if !json_data.is_empty() {
                        let _ = tx.send(Ok(Event::default().data(json_data))).await;
                    }
                }
            }
        }

        let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
    });

    rx
}

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
        .route("/v1/chat/completions", post(chat_completions))
        .route("/models", get(models))
        .route("/v1/models", get(models))
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
    println!("Server running on {}", addr);

    // 修改服务器启动代码
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 处理聊天完成请求
async fn chat_completions(
    headers: HeaderMap,
    request: Request<Body>,
    // Json(chat_request): Json<ChatRequest>,
) -> Result<Response, StatusCode> {
    // 提取并打印原始请求体
    const MAX_BODY_SIZE: usize = 20 * 1024 * 1024;

    let bytes = match axum::body::to_bytes(request.into_body(), MAX_BODY_SIZE).await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!("读取请求体失败: {}", err);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // 打印原始请求体
    if let Ok(body_str) = String::from_utf8(bytes.to_vec()) {
        tracing::info!("原始请求体: {}", body_str);
    }

    // 尝试解析 JSON
    let chat_request: ChatRequest = match serde_json::from_slice(&bytes) {
        Ok(req) => req,
        Err(err) => {
            tracing::error!("JSON解析失败: {}", err);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // 验证认证
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut auth_token = auth_header.replace("Bearer ", "");

    // 验证o1模型不支持流式输出
    if chat_request.model.starts_with("o1-") && chat_request.stream {
        return Err(StatusCode::BAD_REQUEST);
    }
    tracing::info!("chat_request: {:?}", chat_request);

    // 处理多个密钥
    if auth_token.contains(',') {
        auth_token = auth_token.split(',').next().unwrap().trim().to_string();
    }

    if auth_token.contains("%3A%3A") {
        auth_token = auth_token
            .split("%3A%3A")
            .nth(1)
            .unwrap_or(&auth_token)
            .to_string();
    }

    // 格式化消息
    // let formatted_messages = chat_request
    //     .messages
    //     .iter()
    //     .map(|msg| format!("{}:{}", msg.role, msg.content))
    //     .collect::<Vec<_>>()
    //     .join("\n");

    let formatted_messages = chat_request
        .messages
        .iter()
        .map(|msg| {
            let content = msg
                .content
                .iter()
                .map(|part| part.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}:{}", msg.role, content)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 生成请求数据
    let hex_data = string_to_hex(&formatted_messages, &chat_request.model);
    // 准备请求头
    let request_id = Uuid::new_v4();
    let headers = reqwest::header::HeaderMap::from_iter([
        (reqwest::header::CONTENT_TYPE, "application/connect+proto"),
        (reqwest::header::AUTHORIZATION, &format!("Bearer {}", auth_token)),
        // 对于标准 HTTP 头部，使用预定义的常量
        (reqwest::header::HeaderName::from_str("Connect-Accept-Encoding").unwrap(), "gzip,br"),
        (reqwest::header::HeaderName::from_str("Connect-Protocol-Version").unwrap(), "1"),
        (reqwest::header::HeaderName::from_str("User-Agent").unwrap(), "connect-es/1.4.0"),
        (reqwest::header::HeaderName::from_str("X-Amzn-Trace-Id").unwrap(), &format!("Root={}", Uuid::new_v4())),
        (reqwest::header::HeaderName::from_str("X-Cursor-Checksum").unwrap(), "zo6Qjequ9b9734d1f13c3438ba25ea31ac93d9287248b9d30434934e9fcbfa6b3b22029e/7e4af391f67188693b722eff0090e8e6608bca8fa320ef20a0ccb5d7d62dfdef"),
        (reqwest::header::HeaderName::from_str("X-Cursor-Client-Version").unwrap(), "0.42.3"),
        (reqwest::header::HeaderName::from_str("X-Cursor-Timezone").unwrap(), "Asia/Shanghai"),
        (reqwest::header::HeaderName::from_str("X-Ghost-Mode").unwrap(), "false"),
        (reqwest::header::HeaderName::from_str("X-Request-Id").unwrap(), &request_id.to_string()),
        (reqwest::header::HeaderName::from_str("Host").unwrap(), "api2.cursor.sh"),
    ].iter().map(|(k, v)| (
        k.clone(),
        reqwest::header::HeaderValue::from_str(v).unwrap()
    )));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| {
            tracing::error!("创建HTTP客户端失败: {:?}", e);
            tracing::error!(error = %e, "错误详情");

            if let Some(source) = e.source() {
                tracing::error!(source = %source, "错误源");
            }

            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = client
        .post("https://api2.cursor.sh/aiserver.v1.AiService/StreamChat")
        .headers(headers)
        .body(hex_data)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("请求失败: {:?}", e);
            tracing::error!(error = %e, "错误详情");

            // 如果是超时错误
            if e.is_timeout() {
                tracing::error!("请求超时");
            }

            // 如果是连接错误
            if e.is_connect() {
                tracing::error!("连接失败");
            }

            // 如果有请求信息
            if let Some(url) = e.url() {
                tracing::error!(url = %url, "请求URL");
            }

            // 如果有状态码
            if let Some(status) = e.status() {
                tracing::error!(status = %status, "HTTP状态码");
            }

            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if chat_request.stream {
        let mut chunks = Vec::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => chunks.push(chunk),
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }

        let stream = process_stream(chunks).await;
        return Ok(Sse::new(stream).into_response());
    }

    // 非流式响应
    let mut text = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                let res = chunk_to_utf8_string(&chunk);
                if !res.is_empty() {
                    text.push_str(&res);
                }
            }
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    // 清理响应文本
    let re = Regex::new(r"^.*<\|END_USER\|>").unwrap();
    text = re.replace(&text, "").to_string();

    let re = Regex::new(r"^\n[a-zA-Z]?").unwrap();
    text = re.replace(&text, "").trim().to_string();

    let re = Regex::new(r"[\x00-\x1F\x7F]").unwrap();
    text = re.replace_all(&text, "").to_string();

    let response = ChatResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: chat_request.model,
        choices: vec![Choice {
            index: 0,
            message: ResponseMessage {
                role: "assistant".to_string(),
                content: text,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    Ok(Json(response).into_response())
}

// 处理模型列表请求
async fn models() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "object": "list",
        "data": [
            {
                "id": "claude-3-5-sonnet-20241022",
                "object": "model",
                "created": 1713744000,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-3-opus",
                "object": "model",
                "created": 1709251200,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-3.5-haiku",
                "object": "model",
                "created": 1711929600,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-3.5-sonnet",
                "object": "model",
                "created": 1711929600,
                "owned_by": "anthropic"
            },
            {
                "id": "cursor-small",
                "object": "model",
                "created": 1712534400,
                "owned_by": "cursor"
            },
            {
                "id": "gpt-3.5-turbo",
                "object": "model",
                "created": 1677649200,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4",
                "object": "model",
                "created": 1687392000,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4-turbo-2024-04-09",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4o",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4o-mini",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            },
            {
                "id": "o1-mini",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            },
            {
                "id": "o1-preview",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            }
        ]
    }))
}
