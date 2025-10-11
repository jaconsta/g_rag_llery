use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiResponse {
    pub id: String,
    pub object: String,
    #[serde(rename = "created_at")]
    pub created_at: i64,
    pub status: String,
    pub background: bool,
    pub billing: Billing,
    pub error: Value,
    #[serde(rename = "incomplete_details")]
    pub incomplete_details: Value,
    pub instructions: Value,
    #[serde(rename = "max_output_tokens")]
    pub max_output_tokens: Value,
    #[serde(rename = "max_tool_calls")]
    pub max_tool_calls: Value,
    pub model: String,
    pub output: Vec<Output>,
    #[serde(rename = "parallel_tool_calls")]
    pub parallel_tool_calls: bool,
    #[serde(rename = "previous_response_id")]
    pub previous_response_id: Value,
    #[serde(rename = "prompt_cache_key")]
    pub prompt_cache_key: Value,
    pub reasoning: Reasoning,
    #[serde(rename = "safety_identifier")]
    pub safety_identifier: Value,
    #[serde(rename = "service_tier")]
    pub service_tier: String,
    pub store: bool,
    pub temperature: f64,
    pub text: Text,
    #[serde(rename = "tool_choice")]
    pub tool_choice: String,
    pub tools: Vec<Value>,
    #[serde(rename = "top_logprobs")]
    pub top_logprobs: i64,
    #[serde(rename = "top_p")]
    pub top_p: f64,
    pub truncation: String,
    pub usage: Usage,
    pub user: Value,
    pub metadata: Metadata,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Billing {
    pub payer: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub summary: Vec<Value>,
    pub status: Option<String>,
    #[serde(default)]
    pub content: Vec<Content>,
    pub role: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(rename = "type")]
    pub type_field: String,
    pub annotations: Vec<Value>,
    pub logprobs: Vec<Value>,
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reasoning {
    pub effort: String,
    pub summary: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text {
    pub format: Format,
    pub verbosity: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Format {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(rename = "input_tokens")]
    pub input_tokens: i64,
    #[serde(rename = "input_tokens_details")]
    pub input_tokens_details: InputTokensDetails,
    #[serde(rename = "output_tokens")]
    pub output_tokens: i64,
    #[serde(rename = "output_tokens_details")]
    pub output_tokens_details: OutputTokensDetails,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputTokensDetails {
    #[serde(rename = "cached_tokens")]
    pub cached_tokens: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputTokensDetails {
    #[serde(rename = "reasoning_tokens")]
    pub reasoning_tokens: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {}
