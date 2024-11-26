use axum::body::Body;
use axum::extract::Request;
use axum::response::sse::Event;
use axum::Json;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{sse::Sse, IntoResponse, Response},
};
use bytes::Bytes;

use futures::channel::mpsc;
use futures::stream::StreamExt;
use futures::{SinkExt, Stream};
use std::convert::Infallible;
use std::error::Error;
// use http::HeaderName as HttpHeaderName;
use crate::hex_utils::{chunk_to_utf8_string, string_to_hex};
use crate::models;
use regex::Regex;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

// 处理聊天完成请求
pub async fn chat_completions(
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
    let chat_request: models::chat::ChatRequest = match serde_json::from_slice(&bytes) {
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

    let response = models::chat::ChatResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: chat_request.model,
        choices: vec![models::chat::Choice {
            index: 0,
            message: models::chat::ResponseMessage {
                role: "assistant".to_string(),
                content: text,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: models::chat::Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    Ok(Json(response).into_response())
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
                    let var_name = models::chat::StreamResponse {
                        id: response_id.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created: chrono::Utc::now().timestamp(),
                        choices: vec![models::chat::StreamChoice {
                            index: 0,
                            delta: models::chat::Delta {
                                content: text.to_string(),
                            },
                        }],
                    };
                    let response = var_name;

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
