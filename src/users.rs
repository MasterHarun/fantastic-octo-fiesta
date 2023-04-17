use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serenity::model::prelude::{UserId, ChannelId};
use chrono::{DateTime, Utc};

/// # User
/// the user struct contains information about a single user
/// 
/// 
/// ### Fields
/// * `id` - the user id
/// * `settings` - the user settings
/// * `usage` - the user usage
/// 
/// 
/// ### Methods
/// * `new` - creates a new user
/// ---
/// * `modify_settings` - modifies the user settings
/// * `with_settings` - returns a reference to the user settings
/// ---
/// * `modify_usage` - modifies the user usage
/// * `with_usage` - returns a reference to the user usage
/// ---
/// 
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
	pub id: UserId,
	pub settings: UserSettings,
	pub usage: UserUsage,
	// pub command_state: CommandState,
}
impl User {
	pub fn new(id: UserId) -> Self {
		Self {
			id,
			settings: UserSettings::new(),
			usage: UserUsage::new(),
			// command_state: CommandState::None,
		}
	}
	pub fn modify_settings<F>(&mut self, modify: F)
	where
		F: FnOnce(&mut UserSettings),
	{
		modify(&mut self.settings);
	}
	
	pub fn modify_usage<F>(&mut self, modify: F)
	where
		F: FnOnce(&mut UserUsage),
	{
		modify(&mut self.usage);
	}
	pub fn with_settings<F, R>(&self, with_settings: F) -> R
    where
        F: FnOnce(&UserSettings) -> R,
    {
        with_settings(&self.settings)
    }
	pub fn with_usage<F, R>(&self, with_usage: F) -> R
		where
				F: FnOnce(&UserUsage) -> R,
		{
				with_usage(&self.usage)
		}
}


/// # UserSettings
/// the user settings struct contains information about a single user's settings
/// 
/// 
/// ### Fields
/// * `chat_privacy` - the chat privacy setting
/// * `personality` - the personality setting
/// * `model` - the model setting
/// * `command_state` - the command state setting (used for the command system)
/// 
/// 
/// ### Methods
/// * `new` - creates a new UserSettings struct
/// ---
/// * `get_chat_privacy` - returns the chat privacy setting
/// * `set_chat_privacy` - sets the chat privacy setting
/// ---
/// * `get_personality` - returns a reference to the personality setting
/// * `set_personality` - sets the personality setting
/// ---
/// * `get_model` - returns a reference to the model setting
/// * `set_command_state` - sets the command state setting
/// 
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSettings {
	pub chat_privacy: bool,
	pub personality: Personality,
	// the model represents which model is being used for the token usage and limit
	pub model: Model,
	pub command_state: CommandState,
}
impl UserSettings {
	pub fn new() -> Self {
		Self {
			chat_privacy: false,
			personality: Personality::default(),
			model: Model::default(),
			command_state: CommandState::None,
		}
	}
	pub fn get_chat_privacy(&self) -> bool {
		self.chat_privacy
	}
	pub fn set_chat_privacy(&mut self, chat_privacy: bool) {
		self.chat_privacy = chat_privacy;
	}
	pub fn get_personality(&self) -> &Personality {
		&self.personality
	}
	pub fn set_personality(&mut self, personality: Personality) {
		self.personality = personality;
	}
	pub fn get_model(&self) -> &Model {
		&self.model
	}
	pub fn set_command_state(&mut self, command_state: CommandState) {
		self.command_state = command_state;
	}
}


/// # UserUsage
/// the user usage struct contains information about the usage of the user
/// 
/// 
/// ### Fields
/// * `chat_count` - the amount of messages sent by the user in the given channel
/// * `last_chat` - the time of the last message sent by the user in the given channel
/// * `total_tokens` - the total amount of tokens used by the user
/// * `chat_history` - the history of the messages sent by the user in the given channel
/// 
/// 
/// ### Methods
/// * `new` - creates a new UserUsage struct
/// ---
/// * `modify_channel_data` - modifies the channel data of the given channel
/// * `contains_channel` - checks if the user has data for the given channel
/// * `add_channel` - adds a new channel to the user
/// * `reset_channel_usage` - resets the usage of the given channel
/// ---
/// * `increase_chat_count` - increases the chat count by 1
/// * `get_total_tokens` - returns the total amount of tokens used by the user
/// * `add_total_tokens` - adds the given amount of tokens to the total tokens
/// 
/// 
/// ### Usage
/// ```
/// use crate::user::UserUsage;
/// 
/// let mut usage = UserUsage::new();
/// ```
/// 
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUsage {
	pub chat_count: u32,
	pub last_chat: DateTime<Utc>,
	pub total_tokens: u32,
	pub channel_history: FxHashMap<ChannelId, UserChannelData>,

}
impl UserUsage {
	pub fn new() -> Self {
		Self {
			chat_count: 0,
			last_chat: Utc::now(),
			total_tokens: 0,
			channel_history: FxHashMap::default(),
		}
	}
// Method to modify or add a UserChannelData based on ChannelId
pub fn modify_channel_data<F>(&mut self, channel_id: ChannelId, modify: F)
	where
			F: FnOnce(&mut UserChannelData),
	{
		let channel_data = self
			.channel_history
			.entry(channel_id)
			.or_insert(UserChannelData::default());
		modify(channel_data);
	}	
	pub fn contains_channel(&self, channel: ChannelId) -> bool {
		self.channel_history.contains_key(&channel)
	}
	pub fn add_channel(&mut self, channel: ChannelId) {
		self.channel_history.insert(channel, UserChannelData::new(channel));
	}
	// pub fn with_channel_data<F, R>(&mut self, channel: ChannelId, f: F) -> Option<R>
	// 	where
	// 		F: FnOnce(&UserChannelData) -> R,
	// {
	// 	// Get the channel history for the given channel.
	// 	self.channel_history.get(&channel)
	// 		// If the channel history exists, execute the given function on it.
	// 		.map(f)
	// }	
	pub fn reset_channel_usage(&mut self, channel: ChannelId) {
		if let Some(channel_data) = self.channel_history.get_mut(&channel) {
			channel_data.tokens_used = 0;
			channel_data.chat_history.clear();
		}
	}
	
	pub fn get_total_tokens(&self) -> u32 {
		self.total_tokens
	}

	pub fn increase_chat_count(&mut self) {
		self.chat_count += 1;
	}
	pub fn add_total_tokens(&mut self, tokens: u32) {
		self.total_tokens += tokens;
	}
	
}


/// # UserChatHistoryEntry
///  the user chat history entry struct contains information about a single chat message
/// 
/// 
/// ### Fields
/// * `message` - the combined message from the user and the bot
/// * `user_message` - the message sent by the user
/// * `ai_message` - the message sent by the bot
/// * `timestamp` - the time the message was sent
/// * `tokens_amount` - the amount of tokens used by the message
/// * `user_tokens` - the amount of tokens used by the user
/// * `completion_tokens` - the amount of tokens used by the bot
/// 
/// 
/// ### Methods
/// * `new` - creates a new UserChatHistoryEntry struct
/// * `get_user_message` - returns a reference to the user message
/// * `get_ai_message` - returns a reference to the ai message
/// * `get_total_tokens` - returns the total tokens used by the message
/// 
/// 
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserChatHistoryEntry {
	pub message: String, // the combined message from the user and the bot
	pub user_message: String, // the message sent by the user
	pub ai_message: String, // the message sent by the bot
	pub timestamp: DateTime<Utc>,
	pub total_tokens: u32,
	pub user_tokens: u32,
	pub completion_tokens: u32,
}

impl UserChatHistoryEntry {
	pub fn new(
		message: String,
		user_message: String, 
		ai_message: String, 
		total_tokens: u32,
		user_tokens: u32,
		completion_tokens: u32,
	) -> Self {
		Self {
			message,
			user_message,
			ai_message,
			timestamp: Utc::now(),
			total_tokens,
			user_tokens,
			completion_tokens,
		}
	}

	pub fn get_user_message(&self) -> Option<&String> {
		if self.user_message.is_empty() {
			None
		} else {
			Some(&self.user_message)
		}
	}
	pub fn get_ai_message(&self) -> Option<&String> {
		if self.ai_message.is_empty() {
			None
		} else {
			Some(&self.ai_message)
		}
	}
	pub fn get_total_tokens(&self) -> u32 {
		self.total_tokens
	}
}


/// # UserChannelData
/// the UserChannelData struct contains the data for a specific channel
/// 
/// 
/// ### Fields
/// * `channel_id` - the id of the channel
/// * `tokens_used` - the amount of tokens used in the channel
/// * `chat_history` - the chat history of the channel
/// 
/// 
/// ### Methods
/// * `new` - creates a new UserChannelData struct
/// * `default` - returns the default UserChannelData struct
/// * `add_chat_history_entry` - adds a chat history entry to the chat history
/// * `remove_oldest_entry` - removes the oldest entry from the chat history
/// * `get_tokens_used` - returns the amount of tokens used in the channel
/// * `add_tokens_used` - adds tokens to the tokens used
/// 
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserChannelData {
	pub channel_id: ChannelId,
	pub tokens_used: u32,
	pub chat_history: Vec<UserChatHistoryEntry>,
}
impl UserChannelData {
	pub fn new(channel_id: ChannelId) -> Self {
		Self {
			channel_id,
			tokens_used: 0,
			chat_history: Vec::new(),
		}
	}
	pub fn default() -> Self {
		Self {
			channel_id: ChannelId(0),
			tokens_used: 0,
			chat_history: Vec::new(),
		}
	}
	pub fn add_chat_history_entry(&mut self, entry: UserChatHistoryEntry) {
		self.add_tokens_used(entry.total_tokens);
		debug!("total channel tokens used: {}", self.tokens_used);
		self.chat_history.push(entry);
		debug!("channel chat history length: {}", self.chat_history.len());
	}
	pub fn remove_oldest_entry(&mut self) {
		self.tokens_used -= self.chat_history[0].total_tokens;
		self.chat_history.remove(0);
	}
	pub fn get_tokens_used(&self) -> &u32 {
		&self.tokens_used
	}
	pub fn add_tokens_used(&mut self, tokens: u32) {
		self.tokens_used += tokens;
	}
}



/// # Model
/// the Model enum contains the different models that can be used
/// 
/// 
/// ### Fields
/// * `Gpt3_5` - the GPT-3.5 model
/// * `Gpt4` - the GPT-4 model
/// 
/// 
/// ### Methods
/// * `get_token_limit` - returns the token limit of the model
/// 
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Model {
	Gpt3_5 {
		name: String,
		token_limit: u32,
	},
	Gpt4,
}
// todo: add the token limit 
impl Model {
	pub fn default() -> Self {
		Self::Gpt3_5 {
			name: "gpt-3.5-turbo".to_string(),
			token_limit: 4096,
		}
	}
	pub fn get_name(&self) -> String {
		match self {
			Model::Gpt3_5 { name, .. } => name.clone(),
			Model::Gpt4 => "GPT-4".to_string(),
		}
	}
	pub fn get_token_limit(&self) -> &u32 {
		match self {
			Model::Gpt3_5 { token_limit, .. } => token_limit,
			Model::Gpt4 => &8000,
		}
	}

}

/// # Personality
/// the Personality struct contains the different personalities that can be used
/// 
/// 
/// ### Fields
/// * `name` - the name of the personality
/// * `prompt` - the prompt that is sent to the model
/// * `tokens` - the amount of tokens that the personality uses
/// 
/// 
/// ### Methods
/// * `new` - creates a new Personality struct
/// * `default` - returns the default personality
/// 
/// 
/// # Usage
/// ```
/// use crate::user::Personality;
/// 
/// let personality = Personality::new("default".to_string(), "You are a helpful assistant.".to_string(), 0);
/// ```
/// 
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Personality {
	pub name: String,
	pub prompt: String,
	pub tokens: u64,
}
impl Personality {
	pub fn new(name: String, prompt: String, tokens: u64) -> Self {
		Self {
			name,
			prompt,
			tokens,
		}
	}
	pub fn default() -> Self {
		Self {
			name: "default".to_string(),
			prompt: "You are a helpful assistant.".to_string(),
			tokens: 0,
		}
	}
	
}

/// # CommandState
/// the CommandState enum contains the different states that the bot can be in
/// 
/// 
/// ### Fields
/// * `None` - the bot is not in a command state
/// * `PersonalityCommandState` - the bot is in a personality command state - contains the name of the personality that is being edited
/// 
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandState {
	None,
	PersonalityCommandState(String),
}
