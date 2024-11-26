use axum::Json;

// 处理模型列表请求
pub async fn models() -> Json<serde_json::Value> {
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
