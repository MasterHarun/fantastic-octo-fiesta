use serde::Deserialize;

pub struct InteractionData {
  pub interaction_id: String,
  pub response_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
  pub choices: Option<Vec<Choice>>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
  pub message: Message,
}

#[derive(Debug, Deserialize)]
pub struct Message {
  pub content: String,
}
