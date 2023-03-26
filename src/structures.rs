use serde::Deserialize;


// Define a handler struct with a Mutex to store the chat history
pub struct InteractionData {
  pub interaction_id: String,
  pub response_token: String,
}

// Deserialize the API response into these structs
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
