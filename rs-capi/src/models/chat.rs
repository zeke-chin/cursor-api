use serde::Deserializer;
use serde::{Deserialize, Serialize};

// 定义请求模型
#[derive(Debug, Deserialize)]
pub struct Message {
    pub role: String,
    #[serde(deserialize_with = "deserialize_content")]
    pub content: Vec<ContentPart>,
}

// 添加一个辅助枚举
#[derive(Deserialize)]
#[serde(untagged)]
pub enum SingleOrVec<T> {
    Single(T),
    Vec(Vec<T>),
}

// 新增一个字符串或ContentPart的枚举
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ContentItem {
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
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Deserialize)]
pub struct ImageUrl {
    pub url: String,
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
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub stream: bool,
}

// 定义响应模型
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub index: i32,
    pub message: ResponseMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Serialize)]
pub struct StreamResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Serialize)]
pub struct StreamChoice {
    pub index: i32,
    pub delta: Delta,
}

#[derive(Debug, Serialize)]
pub struct Delta {
    pub content: String,
}

// ... existing code ...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_deserialization() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello"
                },
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "Look at this image"
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": "https://example.com/image.jpg"
                            }
                        }
                    ]
                }
            ],
            "stream": true
        }"#;

        let request: ChatRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.model, "gpt-4");
    }

    #[test]
    fn test_chat_request_deserialization2() {
        let json = r#"{
    "model": "gpt-4",
    "stream": true,
    "messages": [
        {
            "content": "你是谁",
            "role": "user"
        },
        {
            "content": "我是一个智能编程助手，旨在帮助你解答与编码相关的问题。如果你有任何编程方面的疑问，请随时问我！",
            "role": "assistant"
        },
        {
            "content": "你是谁",
            "role": "user"
        },
        {
            "content": "我是一个智能编程助手，专注于帮助你解决编程相关的问题。如果你有任何代码方面的疑问或需要帮助，请告诉我！",
            "role": "assistant"
        },
        {
            "content": [
                {
                    "text": "这张图片里面有什么",
                    "type": "text"
                },
                {
                    "image_url": {
                        "url": "data:image/png;base64,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"
                    },
                    "type": "image_url"
                }
            ],
            "role": "user"
        }
    ]
}"#;

        let request: ChatRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.model, "gpt-4");
    }
}
