use serde_json::json;

use crate::{errors::LlmRetrievalError, llm_messages::OpenAiResponse};

pub enum ImagePrompt {
    Description,
    Tags,
}

impl ImagePrompt {
    pub fn to_prompt(&self) -> String {
        match self {
            ImagePrompt::Description => "What is in this image?",
            ImagePrompt::Tags => {
                "Please provide a list of tags in the format of comma sepparated values for this image. Make it not more than twenty please the expected format `tag1,tag2,tag3`"
            }
        }.into()
    }
}

pub async fn fetch_description(
    img_url: &str,
    prompt: ImagePrompt,
) -> Result<String, LlmRetrievalError> {
    // let csv_prompt = "Please provide a list of tags in the format of comma sepparated values for this image. Make it not more than twenty please the expected format `tag1,tag2,tag3`";
    // let description_prompt = "What is in this image?";
    let body_json = json!({
        "model": "gpt-5-mini",
        "input": [
            {
                "role": "user",
                "content": [
                    {"type": "input_text", "text": prompt.to_prompt()},
                    {
                        "type": "input_image",
                        "image_url": img_url
                    },
                ]
            }
        ]
    });
    let openai_key = env!("OPENAI_API_KEY");
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/responses")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", openai_key))
        .json(&body_json)
        .send()
        .await
        .map_err(|e| {
            log::error!("OpenAI query error {e:#?}");
            LlmRetrievalError::OpenAi
        })?;

    let body = resp.json::<OpenAiResponse>().await.map_err(|e| {
        log::error!("OpenAI parse {e:#?}");
        LlmRetrievalError::OpenAi
    })?;

    let response = match body
        .output
        .iter()
        .find(|p| p.status == Some("completed".to_string()))
    {
        Some(r) => r.content.iter().map(|c| c.text.clone()),
        None => return Err(LlmRetrievalError::NoContent),
    };

    Ok(response.collect::<Vec<String>>().join(".\n"))
}
