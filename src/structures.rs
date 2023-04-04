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

pub struct Config {
	pub api_key: String,
	pub discord_token: String,
	pub app_id: String,
	pub rust_log: String,
	pub global_log: String,
}
impl Config {
		pub fn new(api_key: String, discord_token: String, app_id: String, rust_log: String, global_log: String) -> Self {
			Self {
				api_key,
				discord_token,
				app_id,
				rust_log,
				global_log
			}
	}
}