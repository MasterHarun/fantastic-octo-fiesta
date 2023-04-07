 //! Define `Handler` struct and implement the `EventHandler` trait for it
  //!
  //! ## Interaction handling
  //!
  //! Implement the `interaction_create` method to handle incoming interactions
  //! and delegate command handling to the appropriate functions from the `commands` module.
  //!

	use rustc_hash::FxHashMap;
	use std::sync::{Arc, Mutex};

use serenity::{
  async_trait,
  http::Http,
  model::{
    gateway::Ready,
    id::{ChannelId, UserId},
    prelude::interaction::Interaction,
  },
  prelude::{Context, EventHandler},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::commands::*;
use crate::structures::ConfigStruct;
use crate::utils::{acknowledge_interaction, register_application_commands};

pub struct HandlerStruct {
  chat_histories: Arc<Mutex<FxHashMap<(UserId, ChannelId), String>>>,
	// user_usage: Arc<Mutex<HashMap<UserId, Usage>>>,
  chat_privacy: Arc<Mutex<FxHashMap<UserId, bool>>>,
	personas: Arc<Mutex<FxHashMap<String, String>>>,
	user_persona: Arc<Mutex<FxHashMap<(UserId, ChannelId), String>>>,
	config: Arc<ConfigStruct>,
}
// #[async_trait]
pub trait Handler {
	fn new(config: Arc<ConfigStruct>) -> Self;

	fn chat_histories(&self) -> Arc<Mutex<FxHashMap<(UserId, ChannelId), String>>>;
	fn get_chat_history(&self, user: (UserId, ChannelId)) -> String;
	fn update_chat_history(&self, user: (UserId, ChannelId), input: &str, response: &str) -> String;
	fn truncate_chat_history(&self, user: (UserId, ChannelId), input_length: usize) -> String;

	fn chat_privacy(&self) -> Arc<Mutex<FxHashMap<UserId, bool>>>;
	fn get_user_privacy(&self, user_id: UserId) -> bool;

	fn personas(&self) -> Arc<Mutex<FxHashMap<String, String>>>;
	fn get_persona(&self, persona: &str) -> String;
	fn get_user_persona(&self, user: (UserId, ChannelId)) -> String;
	fn set_user_persona(&self, user: (UserId, ChannelId), persona: &str);

	fn get_config(&self) -> Arc<ConfigStruct>;
}

impl Handler for HandlerStruct{
  fn new(config: Arc<ConfigStruct>) -> Self {
    Self {
      chat_histories: Arc::new(Mutex::new(FxHashMap::default())),
      chat_privacy: Arc::new(Mutex::new(FxHashMap::default())),
			personas: Arc::new(Mutex::new(FxHashMap::default())),
			user_persona: Arc::new(Mutex::new(FxHashMap::default())),
			config,
    }
  }

	fn chat_histories(&self) -> Arc<Mutex<FxHashMap<(UserId, ChannelId), String>>> {
		self.chat_histories.clone()
	}

	fn get_chat_history(&self, user: (UserId, ChannelId)) -> String {
		let chat_histories = self.chat_histories.lock().unwrap();
		chat_histories
			.get(&user)
			.unwrap_or(&String::new())
			.clone()
		
	}
	/// Update the chat history for the user and channel
	/// 
	/// # Arguments
	/// 
	/// * `user` - The user's ID and channel ID
	/// * `input` - The user's input
	/// * `response` - The AI's response
	/// 
	/// # Returns
	/// 
	/// * `chat_history` - The updated chat history for the user and channel
	/// 
	fn update_chat_history(&self, user: (UserId, ChannelId), input: &str, response: &str) -> String {
		let mut chat_histories = self.chat_histories.lock().unwrap();
		// if the user and channel key is not in the hashmap, insert a new entry with an empty string
		let chat_history = chat_histories
			.entry(user)
			.or_insert(String::new());
		// add the user's input and the AI's response to the chat history
		// we need to make sure that the history is being properly formatted with ai and user inputs
		// the history should be formatted as follows:
		// user: <user input>
		// ai: <ai response>
		// ...
		// add the user's input
		*chat_history = format!("{}user: {}\n", chat_history, input);
		// add the AI's response
		*chat_history = format!("{}ai: {}\n", chat_history, response);
		
		// truncate the chat history if it is too long
		let tokens: Vec<&str> = chat_history.unicode_words().collect();
		// todo: Depending on the model, the size to truncate will differ
		if tokens.len() > 4096 {
			let input_length = input.unicode_words().count();
			// truncate the chat history
			// the history is truncated by the number of tokens in the input
			self.truncate_chat_history(user, input_length);
			// update the chat history
			*chat_history = self.truncate_chat_history(user, input_length)
		}
		chat_history.clone()
	}
	// before we t
	fn truncate_chat_history(&self, user: (UserId, ChannelId), input_length: usize) -> String {
		// since this is called within the update and the token length is already calculated,
		// we can just use the input_length to truncate the chat history
		let mut chat_histories = self.chat_histories.lock().unwrap();
		let chat_history = chat_histories
			.entry(user)
			.or_insert(String::new());
		let tokens: Vec<&str> = chat_history.unicode_words().collect();
		// truncate the chat history
		// the history is truncated by the number of tokens in the input
		// only the last 4096 tokens are kept
		let truncated_history = tokens
			.iter()
			// skip the first 4096 - input_length tokens
			.skip(tokens.len() - 4096 + input_length)
			.fold(String::new(), |acc, x| format!("{} {}", acc, x));
		// update the chat history
		*chat_history = truncated_history;
		chat_history.clone()
	}


	// this method is used to get the chat_privacy hashmap.
	fn chat_privacy(&self) -> Arc<Mutex<FxHashMap<UserId, bool>>> {
		self.chat_privacy.clone()
	}
	fn get_user_privacy(&self, user_id: UserId) -> bool {
		let privacy = self.chat_privacy.lock().unwrap();
		*privacy.get(&user_id).unwrap_or(&false)
	}

	fn personas(&self) -> Arc<Mutex<FxHashMap<String, String>>> {
		self.personas.clone()
	}

	fn get_persona(&self, persona: &str) -> String {
		let personas = self.personas.lock().unwrap();
		personas
			.get(persona)
			.unwrap_or(&String::new())
			.clone()
	}

	fn get_user_persona(&self, user: (UserId, ChannelId)) -> String {
		let user_persona = self.user_persona.lock().unwrap();
		user_persona
			.get(&user)
			.unwrap_or(&String::new())
			.clone()
	}

	fn set_user_persona(&self, user: (UserId, ChannelId), persona: &str) {
		let mut user_persona = self.user_persona.lock().unwrap();
		user_persona.insert(user, persona.to_string());
	}

	fn get_config(&self) -> Arc<ConfigStruct> {
		self.config.clone()
	}
}

#[async_trait]
impl EventHandler for HandlerStruct {
  async fn ready(&self, _: Context, ready: Ready) {
    info!("{} is connected!", ready.user.name);
		let http = Arc::new(Http::new_with_application_id(
			&self.config.discord_token,
			self.config.app_id.parse::<u64>().unwrap(),
		));
    if let Err(e) = register_application_commands(&http).await {
      error!("Error registering application commands: {:?}", e);
    }
  }

/// Handles interaction events
///
/// Processes the user's interaction with the bot and executes the corresponding command.
///
/// # Arguments
///
/// * `ctx` - The Serenity Context for the event
/// * `interaction` - The Interaction data
/// 
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(command) = interaction {
      let user_id = command.user.id;
      let is_private = *self
        .chat_privacy
        .lock()
        .unwrap()
        .get(&user_id)
        .unwrap_or(&false);

      let ephemeral = match command.data.name.as_str() {
        "private" | "public" => true,
        _ => is_private,
      };

      let interaction_data = match acknowledge_interaction(&command, &ctx, ephemeral).await {
        Ok(data) => data,
        Err(_) => return,
      };

      match command.data.name.as_str() {
        "chat" => {
          chat_command(
            self,
            &ctx,
            &command,
            &interaction_data,
          )
          .await
        }
				"prompt" => {todo!()},
				"personality" => personality_command(self, &ctx, &command ).await,
        "reset" => reset_command(self, &ctx, &command ).await,
        "private" => private_command(&self.chat_privacy, &ctx, &command, &interaction_data).await,
        "public" => public_command(&self.chat_privacy, &ctx, &command, &interaction_data).await,
        _ => {
          error!("Unknown command: {}", command.data.name);
        }
      }
    }
  }
}
