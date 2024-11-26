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
