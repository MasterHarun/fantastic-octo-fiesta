use serde::{Deserialize, Serialize};

/// # ApitRequestBody
/// 
/// A struct holding the request body for the OpenAI API's completion endpoint.
/// 
/// This struct is used to make requests to the OpenAI API's completion endpoint.
/// 
/// ### Fields
/// 
/// * `model` - The model to use for the completion.
/// * `messages` - A vector of `Message`s containing the prompt and completion candidates.
/// * `max_tokens` - The maximum number of tokens to generate.
/// * `temperature` - The temperature to use for the completion.
/// * `user` - The user ID of the user making the request.
/// 
#[derive(Clone, Debug, Serialize)]
pub struct ApiRequestBody {
	pub model: String,
	pub messages: Vec<Message>,
	pub max_tokens: u32,
	pub temperature: f32,
	pub user: String,
}

/// A struct holding the response from the OpenAI API's completion endpoint.
/// 
/// This struct is returned by the OpenAI API's completion endpoint.
/// For more information, see the [OpenAI API documentation](https://beta.openai.com/docs/api-reference/completions/create).
/// 
/// ### fields
/// 
/// * `id` - The ID of the completion.
/// * `object` - The type of object.
/// * `created` - The time the completion was created.
/// * `choices` - A vector of `ChoiceStruct`s containing the completion candidates.
/// * `usage` - A `UsageStruct` containing the usage statistics for the OpenAI API's completion endpoint.
/// 
/// # Methods
/// 
/// * `choices` - Returns a vector of `ChoiceStruct`s containing the completion candidates.
/// * `usage` - Returns a `UsageStruct` containing the usage statistics for the OpenAI API's completion endpoint.
/// 
/// # ExampleS
#[derive(Clone, Debug, Deserialize)]
pub struct ApiResponseStruct {
	pub id: String,
	pub object: String,
	pub created: u64,
	pub choices: Vec<ChoiceStruct>,
	pub usage: UsageStruct,
}

pub trait ApiResponse {
	fn choices(&self) -> Vec<ChoiceStruct>;
	fn usage(&self) -> UsageStruct;
}
impl ApiResponse for ApiResponseStruct {
	fn choices(&self) -> Vec<ChoiceStruct> {
		self.choices.clone()
	}
	fn usage(&self) -> UsageStruct {
		self.usage.clone()
	}
}

/// A struct containing the usage statistics for the OpenAI API's completion endpoint.
/// 
/// This struct is returned by the OpenAI API's completion endpoint.
/// For more information, see the [OpenAI API documentation](https://beta.openai.com/docs/api-reference/completions/create).
/// 
/// ### Fields
/// 
/// * `prompt_tokens` - The number of tokens in the prompt.
/// * `completion_tokens` - The number of tokens in the completion.
/// * `total_tokens` - The total number of tokens in the prompt and completion.
/// 
/// ### Methods
/// 
/// * `prompt_tokens` - Returns the number of tokens in the prompt.
/// * `completion_tokens` - Returns the number of tokens in the completion.
/// * `total_tokens` - Returns the total number of tokens in the prompt and completion.
/// 
/// ### Example
#[derive(Clone, Debug, Deserialize)]
pub struct UsageStruct {
	// pub chat_history: String,
	pub prompt_tokens: u32,
	pub completion_tokens: u32,
	pub total_tokens: u32,
}
pub trait Usage {
	fn prompt_tokens(&self) -> u32;
	fn completion_tokens(&self) -> u32;
	fn total_tokens(&self) -> u32;
}
impl Usage for UsageStruct {
	fn prompt_tokens(&self) -> u32 {
		self.prompt_tokens
	}
	fn completion_tokens(&self) -> u32 {
		self.completion_tokens
	}
	fn total_tokens(&self) -> u32 {
		self.total_tokens
	}
}

/// A struct containing the choices for the OpenAI API's completion endpoint.
/// 
/// This struct is returned by the OpenAI API's completion endpoint.
/// For more information, see the [OpenAI API documentation](https://beta.openai.com/docs/api-reference/completions/create).
/// 
/// ### Fields
/// 
/// * `text` - The text of the choice.
/// * `index` - The index of the choice.
/// * `logprobs` - The log probabilities for the choice.
/// * `finish_reason` - The reason the choice was finished.
/// 
/// ### Methods
/// 
/// * `text` - Returns the text of the choice.
/// * `index` - Returns the index of the choice.
/// * `logprobs` - Returns the log probabilities for the choice.
/// * `finish_reason` - Returns the reason the choice was finished.
/// 
/// ### Example
#[derive(Clone, Debug, Deserialize)]
pub struct ChoiceStruct {
	pub index: u32,
  pub message: Message,
	pub logprobs: Option<LogprobsStruct>,
	pub finish_reason: String,
}

pub trait Choice {
	fn index(&self) -> u32;
	fn message(&self) -> Message;
	fn logprobs(&self) -> LogprobsStruct;
	fn finish_reason(&self) -> String;
}
impl Choice for ChoiceStruct {
	fn index(&self) -> u32 {
		self.index
	}
	fn message(&self) -> Message {
		Message {
			role: self.message.role.clone(),
			content: self.message.content.clone(),
		}
	}
	fn logprobs(&self) -> LogprobsStruct {
		self.logprobs.clone().unwrap()
	}
	fn finish_reason(&self) -> String {
		self.finish_reason.clone()
	}
}

/// A Struct containing the content and role of a message.
/// 
/// This struct is used by the OpenAI API's completion endpoint.
/// 
/// ### Fields
/// 
/// * `role` - The role of the message. Either the user or AI.
/// * `content` - The content of the message.
/// 
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
	pub role: String,
	pub content: String,
}

/// A struct containing the log probabilities for the OpenAI API's completion endpoint.
/// 
/// This struct is returned by the OpenAI API's completion endpoint.
/// For more information, see the [OpenAI API documentation](https://beta.openai.com/docs/api-reference/completions/create).
/// 
/// ### Fields
/// 
/// * `token_logprobs` - The log probabilities for each token.
/// * `top_logprobs` - The log probabilities for each token.
/// * `text_offset` - The offset of each token in the text.
/// 
/// ### Methods
/// 
/// * `token_logprobs` - Returns the log probabilities for each token.
/// * `top_logprobs` - Returns the log probabilities for each token.
/// * `text_offset` - Returns the offset of each token in the text.
/// 
/// ### Example
#[derive(Clone, Debug, Deserialize)]
pub struct LogprobsStruct {
	pub token_logprobs: Option<Vec<Vec<f32>>>,
	pub top_logprobs: Option<Vec<Vec<f32>>>,
	pub text_offset: Option<Vec<Vec<u32>>>,
}

pub trait Logprobs {
	fn token_logprobs(&self) -> Vec<Vec<f32>>;
	fn top_logprobs(&self) -> Vec<Vec<f32>>;
	fn text_offset(&self) -> Vec<Vec<u32>>;
}
impl Logprobs for LogprobsStruct {
	fn token_logprobs(&self) -> Vec<Vec<f32>> {
		self.token_logprobs.clone().unwrap()
	}
	fn top_logprobs(&self) -> Vec<Vec<f32>> {
		self.top_logprobs.clone().unwrap()
	}
	fn text_offset(&self) -> Vec<Vec<u32>> {
		self.text_offset.clone().unwrap()
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigStruct {
	pub api_key: String,
	pub discord_token: String,
	pub app_id: String,
	pub rust_log: String,
	pub global_log: String,
}
pub trait Config {
	fn new(api_key: String, discord_token: String, app_id: String, rust_log: String, global_log: String) -> Self;
	fn api_key(&self) -> String;
	fn discord_token(&self) -> String;
	fn app_id(&self) -> String;
	fn rust_log(&self) -> String;
	fn global_log(&self) -> String;
}
impl Config for ConfigStruct {
		fn new(api_key: String, discord_token: String, app_id: String, rust_log: String, global_log: String) -> Self {
			Self {
				api_key,
				discord_token,
				app_id,
				rust_log,
				global_log
			}
	}
	fn api_key(&self) -> String {
		self.api_key.clone()
	}
	fn discord_token(&self) -> String {
		self.discord_token.clone()
	}
	fn app_id(&self) -> String {
		self.app_id.clone()
	}
	fn rust_log(&self) -> String {
		self.rust_log.clone()
	}
	fn global_log(&self) -> String {
		self.global_log.clone()
	}
}
